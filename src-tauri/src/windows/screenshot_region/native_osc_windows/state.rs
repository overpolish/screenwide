// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows twin of `native_osc_macos/state.rs`. macOS needs a C ABI
//! because its native side is Obj-C; here both sides are Rust, so the same
//! function set is exposed directly and the surface calls replace the FFI
//! calls one for one. Names are kept identical so later stages can be diffed
//! against the macOS file.
//!
//! Every mutator that macOS applied to `screenwide_region_osc_surfaces()`
//! applies to the anchor surface and its peers here.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tauri::{Emitter, EventTarget, Manager, WebviewWindow};
use windows::Win32::{
  Foundation::HWND,
  UI::{
    Input::KeyboardAndMouse::SetFocus,
    WindowsAndMessaging::{PostMessageW, SetTimer},
  },
};

use super::desktop::{global_committed, peer_plan, MonitorProbe, PeerPlan};
use super::input;
use super::ruler::{self, RulerData};
use super::surface::{self, Gpu, Surface};
use crate::osc::{
  controller::RegionController,
  desktop::{DesktopBinding, DesktopDisplay},
  geometry::{Monitor, Point, Rect, Size},
  protocol::{OscResult, Purpose},
  runtime::OscRuntime,
  scene::{RegionScene, RegionSceneOwner},
  style::overlay_palette,
};
use crate::text_recognition::visual::OcrRectPacket;

/// `WM_DISPLAYCHANGE` reaches every top-level window, so a topology change
/// arrives once per peer. Only the first of each burst is acted on.
const LAYOUT_COALESCE: Duration = Duration::from_millis(250);

/// The displays and anchor a binding was last built from. `configure_desktop`
/// compares against it so `layout_changed` is true on the first configure and
/// on any topology change, the way AppKit's `layout_matches` did.
type LayoutSnapshot = (Vec<DesktopDisplay>, u32);

/// The anchor surface plus one peer per non-anchor display. macOS peers shared
/// the root's `rustContext` with a NULL release; here the whole set lives in
/// one context and the runtime is owned by that context alone.
pub(crate) struct SurfaceSet {
  root: Surface,
  peers: Vec<Surface>,
}

impl SurfaceSet {
  pub(crate) fn root_mut(&mut self) -> &mut Surface {
    &mut self.root
  }

  pub(crate) fn all_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
    std::iter::once(&mut self.root).chain(self.peers.iter_mut())
  }

  pub(crate) fn find_mut(&mut self, hwnd: HWND) -> Option<&mut Surface> {
    self.all_mut().find(|surface| surface.hwnd() == hwnd)
  }

  /// The window every session-wide ruler timer is hung on. The root surface
  /// always exists, so this never has to fall back.
  pub(crate) fn root_hwnd(&self) -> HWND {
    self.root.hwnd()
  }

  fn for_display_mut(&mut self, display_id: u32) -> Option<&mut Surface> {
    self
      .all_mut()
      .find(|surface| surface.display_id == display_id)
  }

  /// Port of `screenwide_region_osc_apply_region`: one desktop-global rect,
  /// applied to every surface, each subtracting its own offset when it draws.
  fn apply_region(&mut self, region: Rect, visible: bool) {
    for surface in self.all_mut() {
      surface.set_region(region, visible);
    }
  }

  /// True when the live peers already match the plan, so nothing is rebuilt.
  fn peers_match(&self, plan: &[PeerPlan]) -> bool {
    self.peers.len() == plan.len()
      && self.peers.iter().zip(plan).all(|(peer, planned)| {
        peer.display_id == planned.display_id
          && peer.peer_geometry() == Some((planned.bounds, planned.scale))
      })
  }
}

pub(crate) struct Context {
  /// The runtime the macOS side owned as a raw `Box`; here the registry owns
  /// it and drops it with the context.
  runtime: Box<OscRuntime>,
  gpu: Arc<Gpu>,
  host: HWND,
  pub(crate) surfaces: Mutex<SurfaceSet>,
  layout: Mutex<Option<LayoutSnapshot>>,
  probes: Mutex<Vec<MonitorProbe>>,
  /// Set when a configure found a *different* desktop than the one already
  /// bound. Establishing the first binding is not a topology change, so it
  /// never notifies — AppKit only fired the callback from its screen-parameters
  /// notification.
  pending_layout_notice: Mutex<bool>,
  layout_notified: Mutex<Option<Instant>>,
  /// The gesture and latched-key state the macOS side kept on the root
  /// `ScreenwideRegionOSC`; it is per session, not per surface.
  pub(crate) ruler: Mutex<RulerSession>,
}

/// Ruler interaction state shared by every surface of one session.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RulerSession {
  /// A label drag owns every subsequent drag and up until it ends.
  pub label_drag_active: bool,
  /// The last middle-button pan sample, in surface-local points.
  pub pan_last: Option<Point>,
  /// Held keys latch until their key-up fires the release phase
  /// (`+input.m:440-471`).
  pub range_key: u16,
  pub guide_key: u16,
  pub radius_key: u16,
}

impl RulerSession {
  pub(crate) fn latched(&self) -> bool {
    self.range_key != 0 || self.guide_key != 0 || self.radius_key != 0
  }
}

// The context is reached from the UI thread through the registry; the host
// handle it keeps is an opaque process-wide token and every surface it owns is
// guarded by the surface mutex.
unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
  fn input(&self, phase: u32, point: Point, modifiers: u8) -> OscResult {
    self.runtime.input(phase, point, modifiers)
  }

  pub(crate) fn is_ruler(&self) -> bool {
    self.runtime.purpose == Purpose::Ruler
  }

  pub(crate) fn is_text_recognition(&self) -> bool {
    self.runtime.purpose == Purpose::TextRecognition
  }

  /// `native_osc_ruler_label_input` (`native_osc_macos/state.rs:326`), reached
  /// directly because both sides of this port are Rust.
  pub(crate) fn ruler_label_input(
    &self,
    operation: u32,
    kind: u8,
    id: u64,
    pointer: Point,
    label_center: Point,
  ) -> OscResult {
    catch_unwind(AssertUnwindSafe(|| {
      self
        .runtime
        .ruler_label_input(operation, kind, id, pointer, label_center)
    }))
    .unwrap_or_else(|_| crate::osc::runtime::invalid_result())
  }

  /// `native_osc_ruler_viewport_input` (`:288`). The macOS wrapper returned a
  /// "handled" flag; here the caller reads `status != Invalid` for the same
  /// answer, which is exactly what that flag was computed from.
  pub(crate) fn ruler_viewport_input(
    &self,
    display_id: u32,
    operation: u32,
    anchor: Point,
    delta: Point,
  ) -> OscResult {
    catch_unwind(AssertUnwindSafe(|| {
      self
        .runtime
        .ruler_viewport_input(display_id, operation, anchor, delta)
    }))
    .unwrap_or_else(|_| crate::osc::runtime::invalid_result())
  }
}

/// The per-window slot is an inner `OnceLock` handed out from under the map
/// lock rather than a `Result` stored in the map: creating a surface
/// round-trips to the window's event-loop thread and must not run while a lock
/// that thread could also want is held. A failure stays cached so a broken GPU
/// is not retried on every frontend update.
type ContextSlot = Arc<OnceLock<Result<Arc<Context>, String>>>;

static CONTEXTS: OnceLock<Mutex<HashMap<isize, ContextSlot>>> = OnceLock::new();
/// Reverse lookup for the window procedures, which are handed only their own
/// HWND. Every surface window of a context is registered here.
static BY_SURFACE: OnceLock<Mutex<HashMap<isize, Arc<Context>>>> = OnceLock::new();

fn contexts() -> &'static Mutex<HashMap<isize, ContextSlot>> {
  CONTEXTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn by_surface() -> &'static Mutex<HashMap<isize, Arc<Context>>> {
  BY_SURFACE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn context_for_surface(hwnd: HWND) -> Option<Arc<Context>> {
  let map = by_surface().lock().ok()?;
  map.get(&(hwnd.0 as isize)).map(Arc::clone)
}

fn register_surface(hwnd: HWND, context: &Arc<Context>) {
  if let Ok(mut map) = by_surface().lock() {
    map.insert(hwnd.0 as isize, Arc::clone(context));
  }
}

fn unregister_surface(hwnd: HWND) {
  if let Ok(mut map) = by_surface().lock() {
    map.remove(&(hwnd.0 as isize));
  }
}

pub(crate) fn dispatch_input(
  context: &Context,
  phase: u32,
  point: Point,
  modifiers: u8,
) -> OscResult {
  catch_unwind(AssertUnwindSafe(|| context.input(phase, point, modifiers)))
    .unwrap_or_else(|_| crate::osc::runtime::invalid_result())
}

fn host(window: &WebviewWindow) -> Option<HWND> {
  // Tauri hands back its own `windows` binding's handle; the value is the
  // same process-wide token.
  window.hwnd().ok().map(|hwnd| HWND(hwnd.0))
}

fn slot(hwnd: HWND) -> Option<ContextSlot> {
  let mut map = contexts().lock().ok()?;
  Some(Arc::clone(
    map
      .entry(hwnd.0 as isize)
      .or_insert_with(|| Arc::new(OnceLock::new())),
  ))
}

fn attach(window: &WebviewWindow, width: f64, height: f64, purpose: Purpose) -> bool {
  let Some(hwnd) = host(window) else {
    eprintln!("The Windows region OSC found no window handle to attach to");
    return false;
  };
  let Some(slot) = slot(hwnd) else {
    return false;
  };
  let created = slot.get_or_init(|| create(window, hwnd, width, height, purpose));
  created.is_ok()
}

fn create(
  window: &WebviewWindow,
  hwnd: HWND,
  width: f64,
  height: f64,
  purpose: Purpose,
) -> Result<Arc<Context>, String> {
  let overlay = surface::create_on_owning_thread(window, hwnd, None).inspect_err(|error| {
    eprintln!(
      "The Windows region OSC could not attach to {}: {error}",
      window.label()
    )
  })?;
  let gpu = Gpu::new().inspect_err(|error| {
    eprintln!("The Windows region OSC surface could not be created: {error}");
  })?;
  let root = Surface::root(Arc::clone(&gpu), hwnd, overlay).inspect_err(|error| {
    eprintln!("The Windows region OSC surface could not be created: {error}");
  })?;
  let context = Arc::new(Context {
    runtime: OscRuntime::new(window.clone(), width, height, purpose),
    gpu,
    host: hwnd,
    surfaces: Mutex::new(SurfaceSet {
      root,
      peers: Vec::new(),
    }),
    layout: Mutex::new(None),
    probes: Mutex::new(Vec::new()),
    pending_layout_notice: Mutex::new(false),
    layout_notified: Mutex::new(None),
    ruler: Mutex::new(RulerSession::default()),
  });
  register_surface(overlay, &context);
  Ok(context)
}

pub(crate) fn ensure_attached(window: &WebviewWindow, width: f64, height: f64) -> bool {
  with_context(window, |_| ()).is_some() || attach(window, width, height, Purpose::Region)
}

/// The Text Recognition twin: same idempotent attach, `Purpose::TextRecognition`
/// context (`native_osc_macos/state.rs:400`).
pub(crate) fn ensure_text_recognition_attached(
  window: &WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(window, |_| ()).is_some() || attach(window, width, height, Purpose::TextRecognition)
}

/// The Ruler twin: same idempotent attach, `Purpose::Ruler` context
/// (`native_osc_macos/state.rs:410`).
pub(crate) fn ensure_ruler_attached(window: &WebviewWindow, width: f64, height: f64) -> bool {
  with_context(window, |_| ()).is_some() || attach(window, width, height, Purpose::Ruler)
}

pub(crate) fn input_hwnd(window: &WebviewWindow) -> Option<isize> {
  with_context(window, |context| {
    context
      .surfaces
      .lock()
      .ok()
      .map(|set| set.root_hwnd().0 as isize)
  })
  .flatten()
}

/// Gives the nonactivating compositor child keyboard focus after its Tauri
/// host has become foreground. This mirrors macOS making the Ruler panel key
/// and also provides a direct WM_KEYDOWN path alongside the low-level hook.
pub(crate) fn focus_ruler_input(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    let hwnd = context.surfaces.lock().ok().map(|set| set.root_hwnd());
    hwnd.is_some_and(|hwnd| unsafe { SetFocus(Some(hwnd)) }.is_ok())
  })
  .unwrap_or(false)
}

pub(crate) fn with_context<T>(
  window: &WebviewWindow,
  work: impl FnOnce(&Context) -> T,
) -> Option<T> {
  let hwnd = host(window)?;
  let slot = {
    let map = contexts().lock().ok()?;
    Arc::clone(map.get(&(hwnd.0 as isize))?)
  };
  let context = slot.get()?.as_ref().ok()?;
  Some(work(context))
}

/// Runs `work` over the whole surface set; `None` when unattached.
fn with_surfaces<T>(window: &WebviewWindow, work: impl FnOnce(&mut SurfaceSet) -> T) -> Option<T> {
  with_context(window, |context| {
    context.surfaces.lock().ok().map(|mut set| work(&mut set))
  })
  .flatten()
}

/// Updates the top-level desktop peers alongside the host webview's affinity.
/// The root compositor HWND is a child and inherits the webview's capture
/// treatment; `SetWindowDisplayAffinity` only accepts top-level windows.
pub(crate) fn set_capture_affinity(window: &WebviewWindow, capturable: bool) -> bool {
  with_surfaces(window, |set| {
    set.peers.iter().all(|peer| {
      surface::set_capture_affinity(peer.hwnd(), capturable)
        .inspect_err(|error| {
          eprintln!("The Windows region OSC peer could not set capture affinity: {error}")
        })
        .is_ok()
    })
  })
  .unwrap_or(true)
}

pub(crate) fn set_committed(window: &WebviewWindow, rect: Option<Rect>) -> bool {
  with_context(window, |context| {
    context
      .runtime
      .controller
      .lock()
      .map(|mut controller| controller.set_committed(rect))
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

/// Clears the borrowed OSC before its window can be presented for a quick
/// screenshot. The recording region remains in frontend storage and will be
/// synchronized back when the normal Region editor resumes.
pub(crate) fn clear_region(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    if let Ok(mut controller) = context.runtime.controller.lock() {
      let _ = controller.set_committed(None);
    }
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.region = Rect::default();
      scene.visible = false;
    }
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    set.apply_region(Rect::default(), false);
    true
  })
  .unwrap_or(false)
}

pub(crate) fn present_region(window: &WebviewWindow, rect: Option<Rect>) -> bool {
  let rect = rect.unwrap_or_default();
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.region = rect;
      scene.visible = true;
    }
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    set.apply_region(rect, true);
    true
  })
  .unwrap_or(false)
}

pub(crate) fn region_scene(window: &WebviewWindow) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .map(|scene| scene.presented())
  })
  .flatten()
}

pub(crate) fn region_scene_request_base(
  window: &WebviewWindow,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .map(|scene| scene.request_base(owner))
  })
  .flatten()
}

pub(crate) fn reconcile_region_scene_request(
  window: &WebviewWindow,
  requested: RegionScene,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .and_then(|mut state| state.reconcile_request(requested, owner))
  })
  .flatten()
}

pub(crate) fn restore_normal_region_scene(window: &WebviewWindow) -> bool {
  let Some(scene) = with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .and_then(|state| state.normal_presentation())
  })
  .flatten() else {
    return false;
  };
  apply_region_scene(window, scene)
}

/// Applies the portable Region scene to the Windows compositor. The adapter
/// diffs lifecycle-owned fields so a workflow refresh cannot needlessly
/// re-present desktop or snapshot surfaces.
pub(crate) fn apply_region_scene(window: &WebviewWindow, next: RegionScene) -> bool {
  if next.overlay != overlay_palette() {
    eprintln!("The Windows region OSC refused a scene with a foreign overlay palette");
    return false;
  }
  let Some(previous) = with_context(window, |context| {
    let mut scene = context.runtime.scene.lock().ok()?;
    let previous = scene.presented();
    scene.set_presented(next);
    context.runtime.allow_drawing.store(
      next.interaction.allow_drawing,
      std::sync::atomic::Ordering::Relaxed,
    );
    let mut controller = context.runtime.controller.lock().ok()?;
    controller.set_aspect(next.interaction.aspect);
    Some(previous)
  })
  .flatten() else {
    return false;
  };

  if previous.chrome.frame_visible != next.chrome.frame_visible {
    set_show_frame(window, next.chrome.frame_visible);
  }
  if previous.chrome.handles_visible != next.chrome.handles_visible {
    set_show_handles(window, next.chrome.handles_visible);
  }
  if previous.interaction.input_enabled != next.interaction.input_enabled {
    set_input_enabled(window, next.interaction.input_enabled);
  }
  if previous.interaction.exclusion_rect != next.interaction.exclusion_rect {
    set_exclusion_rect(window, next.interaction.exclusion_rect.unwrap_or_default());
  }
  if previous.snapshot.presented != next.snapshot.presented {
    set_snapshot_presented(window, next.snapshot.presented);
  }
  if previous.snapshot.composited != next.snapshot.composited {
    set_snapshot_composited(window, next.snapshot.composited);
  }
  // Geometry is submitted before desktop peers are presented so a newly shown
  // surface can never expose the previous tool's cutout for a frame.
  let presented = with_surfaces(window, |set| {
    set.apply_region(next.region, next.visible);
    true
  })
  .unwrap_or(false);
  if !presented {
    return false;
  }
  if previous.desktop_presented != next.desktop_presented {
    set_desktop_presented(window, next.desktop_presented);
  }
  true
}

/// True when this binding describes a different desktop than the last one.
pub(crate) fn layout_changed(previous: Option<&LayoutSnapshot>, binding: &DesktopBinding) -> bool {
  previous
    .is_none_or(|(displays, anchor)| *anchor != binding.anchor_id || *displays != binding.displays)
}

/// Enumerates the desktop and stamps `layout_changed` from the last binding
/// this context configured.
pub(crate) fn configure_desktop_window(
  window: &WebviewWindow,
  anchor_id: u32,
) -> Result<DesktopBinding, String> {
  let (mut binding, probes) = super::desktop::configure_window(window, anchor_id)?;
  if binding.anchor_id != anchor_id {
    eprintln!(
      "The Windows region OSC substituted monitor {} for the lost anchor {anchor_id}",
      binding.anchor_id
    );
  }
  let changed = with_context(window, |context| {
    if let Ok(mut stored) = context.probes.lock() {
      *stored = probes;
    }
    let Ok(mut layout) = context.layout.lock() else {
      return true;
    };
    let established = layout.is_some();
    let changed = layout_changed(layout.as_ref(), &binding);
    if changed {
      *layout = Some((binding.displays.clone(), binding.anchor_id));
    }
    if let Ok(mut pending) = context.pending_layout_notice.lock() {
      *pending = changed && established;
    }
    changed
  })
  .unwrap_or(true);
  binding.layout_changed = changed;
  Ok(binding)
}

/// Port of `configure_desktop` (`native_osc_macos/state.rs:598`): the desktop
/// union replaces the single monitor as the controller's coordinate space, the
/// anchor's origin becomes the root surface's desktop offset, and the peer
/// windows are rebuilt whenever the topology moved.
pub(crate) fn configure_desktop(
  window: &WebviewWindow,
  binding: DesktopBinding,
  local: Option<Rect>,
) -> bool {
  let Some(configured) = with_context(window, |context| {
    let Some(anchor) = binding.anchor() else {
      return false;
    };
    let committed = global_committed(&binding, local);
    let controller = RegionController::new(binding.virtual_monitor(), committed, None);
    let Ok(mut current_controller) = context.runtime.controller.lock() else {
      return false;
    };
    let Ok(mut desktop) = context.runtime.desktop.lock() else {
      return false;
    };
    if let Ok(mut set) = context.surfaces.lock() {
      let root = set.root_mut();
      root.display_id = binding.anchor_id;
      root.set_desktop_offset(anchor.origin);
    }
    *current_controller = controller;
    *desktop = Some(binding.clone());
    true
  }) else {
    return false;
  };
  if !configured {
    return false;
  }
  sync_peers(window, &binding);
  // The peers exist before the webview hears about the new topology.
  let notify = with_context(window, |context| {
    context
      .pending_layout_notice
      .lock()
      .map(|mut pending| std::mem::replace(&mut *pending, false))
      .unwrap_or(false)
  })
  .unwrap_or(false);
  if notify {
    with_context(window, notify_layout_changed);
  }
  true
}

/// Port of `rebuild_surfaces` (`+desktop.m:113-186`): a topology change tears
/// the peers down — cancelling their gestures, lens and cursor first — and
/// rebuilds them from the new binding.
fn sync_peers(window: &WebviewWindow, binding: &DesktopBinding) {
  let Some(context) = context_arc(window) else {
    return;
  };
  let probes = context
    .probes
    .lock()
    .map(|probes| probes.clone())
    .unwrap_or_default();
  let plan = peer_plan(binding, &probes);
  {
    let Ok(set) = context.surfaces.lock() else {
      return;
    };
    if set.peers_match(&plan) {
      return;
    }
  }
  let capturable = crate::settings::current(window.app_handle()).record_screenwide_windows;
  // Windows must be created on the thread that owns the host, and that thread
  // must not be blocked while a surface lock is held.
  let mut built = Vec::new();
  for planned in &plan {
    let peer =
      surface::create_on_owning_thread(window, context.host, Some((planned.bounds, capturable)))
        .and_then(|hwnd| {
          Surface::peer(
            Arc::clone(&context.gpu),
            hwnd,
            planned.display_id,
            planned.bounds,
            planned.scale,
          )
        });
    match peer {
      Ok(peer) => built.push((peer, planned.offset)),
      Err(error) => {
        eprintln!("The Windows region OSC peer could not be created: {error}");
        return;
      }
    }
  }
  // The registry is only ever touched outside the surface lock, so the two
  // never nest in opposite orders.
  let (retired, adopted) = {
    let Ok(mut set) = context.surfaces.lock() else {
      return;
    };
    let retired = set
      .peers
      .drain(..)
      .map(|peer| peer.hwnd())
      .collect::<Vec<_>>();
    // The rebuild cancels whatever the old peers were doing before the new
    // ones inherit the scene.
    let root = set.root_mut();
    root.release_pointer();
    root.gesture_active = false;
    root.magnifier = None;
    let region = root.region;
    let visible = root.visible;
    let show_frame = root.show_frame;
    let show_handles = root.show_handles;
    let input_enabled = root.input_enabled;
    let desktop_presented = root.desktop_presented;
    let snapshot_presented = root.snapshot_presented;
    let snapshot_composited = root.snapshot_composited;
    let mut adopted = Vec::with_capacity(built.len());
    for (mut peer, offset) in built {
      peer.set_desktop_offset(offset);
      peer.show_frame = show_frame;
      peer.show_handles = show_handles;
      peer.input_enabled = input_enabled;
      peer.desktop_presented = desktop_presented;
      peer.snapshot_presented = snapshot_presented;
      peer.snapshot_composited = snapshot_composited;
      adopted.push((peer.hwnd(), input_enabled));
      peer.set_region(region, visible);
      set.peers.push(peer);
    }
    (retired, adopted)
  };
  for hwnd in retired {
    unregister_surface(hwnd);
  }
  for (hwnd, input_enabled) in adopted {
    register_surface(hwnd, &context);
    surface::set_pointer_passthrough(hwnd, false, !input_enabled);
  }
}

fn context_arc(window: &WebviewWindow) -> Option<Arc<Context>> {
  let hwnd = host(window)?;
  let slot = {
    let map = contexts().lock().ok()?;
    Arc::clone(map.get(&(hwnd.0 as isize))?)
  };
  slot.get()?.as_ref().ok().map(Arc::clone)
}

/// Port of `native_osc_layout_changed` (`native_osc_macos/state.rs:362-382`).
/// Region hosts tell their webview; the other purposes restart their sessions
/// in Rust once stages 3 and 4 land.
pub(crate) fn notify_layout_changed(context: &Context) {
  {
    let Ok(mut last) = context.layout_notified.lock() else {
      return;
    };
    let now = Instant::now();
    if last.is_some_and(|last| now.duration_since(last) < LAYOUT_COALESCE) {
      return;
    }
    *last = Some(now);
  }
  match context.runtime.purpose {
    Purpose::Region => {
      let window = &context.runtime.window;
      let _ = window.emit_to(
        EventTarget::webview_window(window.label()),
        super::NATIVE_OSC_LAYOUT_EVENT,
        (),
      );
    }
    Purpose::TextRecognition => {
      crate::text_recognition::restart_after_topology_change(context.runtime.window.app_handle());
    }
    Purpose::Ruler => {
      crate::ruler::restart_after_topology_change(context.runtime.window.app_handle());
    }
  }
}

/// Called from a peer's window procedure on `WM_DISPLAYCHANGE`/`WM_DPICHANGED`.
pub(crate) fn notify_layout_changed_for_surface(hwnd: HWND) {
  if let Some(context) = context_for_surface(hwnd) {
    notify_layout_changed(&context);
  }
}

pub(crate) fn set_monitor(window: &WebviewWindow, width: f64, height: f64) -> bool {
  with_context(window, |context| {
    context
      .runtime
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_monitor(Monitor {
          size: Size { width, height },
        })
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub(crate) fn set_allow_drawing(window: &WebviewWindow, allow_drawing: bool) -> bool {
  with_context(window, |context| {
    context
      .runtime
      .allow_drawing
      .store(allow_drawing, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.allow_drawing = allow_drawing;
    }
  })
  .is_some()
}

pub(crate) fn set_magnifier_source(
  window: &WebviewWindow,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  with_surfaces(window, |set| {
    set
      .for_display_mut(display_id)
      .is_some_and(|surface| surface.set_magnifier_source(rgba, width, height))
  })
  .unwrap_or(false)
}

pub(crate) fn set_aspect(window: &WebviewWindow, aspect: Option<f64>) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.aspect = aspect;
    }
    context
      .runtime
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_aspect(aspect);
        true
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

/// Port of `screenwide_region_osc_set_input_enabled` (`+state.m:25-76`):
/// disabling while a gesture runs cancels it through the runtime first, so the
/// controller never keeps a half-finished drag across a workflow change.
pub(crate) fn set_input_enabled(window: &WebviewWindow, enabled: bool) -> bool {
  let Some(cancel) = with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.input_enabled = enabled;
    }
    context
      .surfaces
      .lock()
      .map(|mut set| {
        !enabled
          && set
            .all_mut()
            .any(|surface| surface.input_enabled && surface.gesture_active)
      })
      .unwrap_or(false)
  }) else {
    return false;
  };
  if cancel {
    let cancelled = with_context(window, |context| {
      dispatch_input(context, 5, Point::default(), 0)
    });
    if let Some(result) = cancelled {
      if result.ruler_flags & 1 == 0 {
        let region = if result.has_region == 0 {
          Rect::default()
        } else {
          Rect::from_xywh(result.x, result.y, result.width, result.height)
        };
        with_surfaces(window, |set| {
          let visible = set.root_mut().visible;
          set.apply_region(region, visible);
        });
      }
    }
  }
  let Some(surfaces) = with_surfaces(window, |set| {
    let mut surfaces = Vec::with_capacity(set.peers.len() + 1);
    for surface in set.all_mut() {
      // Ready OCR packets refresh the compositor after every changed caret.
      // They repeat `input_enabled = true` and must not end the Win32 mouse
      // capture that owns the current drag. Only disabling input cancels a
      // gesture; a redundant enabled update preserves it.
      surface.gesture_active = gesture_after_input_update(surface.gesture_active, enabled);
      surface.input_enabled = enabled;
      surfaces.push((surface.hwnd(), surface.is_root()));
      if !enabled {
        surface.magnifier = None;
        surface.release_pointer();
      }
      surface.draw();
    }
    surfaces
  }) else {
    return false;
  };
  // Keep native style changes outside the SurfaceSet mutex.
  surfaces
    .iter()
    .all(|(hwnd, is_root)| surface::set_pointer_passthrough(*hwnd, *is_root, !enabled))
}

fn gesture_after_input_update(active: bool, enabled: bool) -> bool {
  active && enabled
}

pub(crate) fn set_show_handles(window: &WebviewWindow, show_handles: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.chrome.handles_visible = show_handles;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.show_handles = show_handles;
      }
    }
  })
  .is_some()
}

pub(crate) fn set_show_frame(window: &WebviewWindow, show_frame: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.chrome.frame_visible = show_frame;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.show_frame = show_frame;
        surface.draw();
      }
    }
  })
  .is_some()
}

/// The exclusion rect is the webview's own toolbar, so it belongs to the
/// anchor surface only (`+state.m:78-88` zeroes it on every peer).
fn set_exclusion_rect(window: &WebviewWindow, rect: Rect) -> bool {
  with_surfaces(window, |set| {
    set.root_mut().exclusion_rect = rect;
    for peer in set.peers.iter_mut() {
      peer.exclusion_rect = Rect::default();
    }
  })
  .is_some()
}

/// Port of `screenwide_region_osc_set_desktop_presented` (`+desktop.m:247-271`):
/// window ordering only. With no peers — the single-monitor case — this stays
/// a scene mirror and nothing is ordered.
pub(crate) fn set_desktop_presented(window: &WebviewWindow, presented: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.desktop_presented = presented;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        if !presented {
          surface.release_pointer();
        }
        // The root ignores the flag when it draws; it is mirrored there only
        // so a peer rebuild can inherit the current presentation.
        surface.desktop_presented = presented;
        surface.draw();
      }
    }
  })
  .is_some()
}

/// Port of `claimPointerSurfaceNow` (`+input.m:141-157`): the surface whose
/// window contains the pointer takes the cursor, and every other one lets go.
pub(crate) fn claim_pointer_surface(window: &WebviewWindow) -> bool {
  let target = with_surfaces(window, |set| {
    let pointer = surface::cursor_position();
    let mut claimed = false;
    let mut target = None;
    if let Some(pointer) = pointer {
      for surface in set.all_mut() {
        if !claimed
          && surface.input_enabled
          && surface.visible
          && surface.contains_screen_point(pointer)
        {
          surface.claim_pointer();
          claimed = true;
          target = Some(surface.hwnd());
        } else {
          surface.release_pointer();
        }
      }
    }
    // With no peers the anchor surface is the only candidate; claiming it
    // keeps the single-monitor path identical to stage 1.
    if !claimed && set.peers.is_empty() {
      set.root_mut().claim_pointer();
      target = Some(set.root_hwnd());
    }
    target
  });
  if let Some(Some(hwnd)) = target {
    // WebView2 can apply its arrow cursor at the end of the same focus/show
    // turn. Reassert ours on the next owner-thread message, mirroring macOS's
    // immediate + dispatch_async cursor claim.
    let _ = unsafe {
      PostMessageW(
        Some(hwnd),
        input::RULER_CURSOR_EVENT,
        windows::Win32::Foundation::WPARAM(0),
        windows::Win32::Foundation::LPARAM(0),
      )
    };
  }
  target.is_some()
}

/// Port of `+snapshot.m:13-20`: the frozen desktop is pushed per display and
/// kept by the surface that owns that display.
pub(crate) fn set_snapshot(
  window: &WebviewWindow,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  with_surfaces(window, |set| {
    set
      .for_display_mut(display_id)
      .is_some_and(|surface| surface.set_snapshot(rgba, width, height))
  })
  .unwrap_or(false)
}

/// Port of `screenwide_region_osc_set_ocr` (`+ocr.m:137-185`). Every surface
/// keeps the share of the highlights that lands on it, and the surface showing
/// most of the selection hosts the status pill or the ready toolbar.
pub(crate) fn set_ocr(
  window: &WebviewWindow,
  phase: u32,
  rects: &[OcrRectPacket],
  message: &str,
) -> bool {
  if phase > 3 {
    eprintln!("The Windows region OSC refused an unknown OCR phase: {phase}");
    return false;
  }
  with_surfaces(window, |set| {
    let mut areas = Vec::new();
    for surface in set.all_mut() {
      let bounds = surface.logical_size();
      let offset = surface.desktop_offset();
      let local = surface.local_rect(surface.region);
      surface.ocr.apply(phase, rects, message, offset, bounds);
      areas.push(super::ocr::overlap_area(local, bounds));
    }
    let target = areas
      .iter()
      .copied()
      .enumerate()
      .filter(|(_, area)| *area > 0.0)
      .max_by(|left, right| left.1.total_cmp(&right.1))
      .map(|(index, _)| index);
    for (index, surface) in set.all_mut().enumerate() {
      surface.ocr.set_target(target == Some(index));
      surface.draw();
    }
    true
  })
  .unwrap_or(false)
}

/// Port of `screenwide_region_osc_ocr_set_cancel_visible` (`+ocr_cancel.m:166`).
#[expect(
  dead_code,
  reason = "the reusable cancel OSC is intentionally not shown by OCR"
)]
pub(crate) fn set_ocr_cancel_visible(window: &WebviewWindow, visible: bool) -> bool {
  with_surfaces(window, |set| {
    for surface in set.all_mut() {
      surface.ocr.set_cancel_visible(visible);
      surface.draw();
    }
  })
  .is_some()
}

/// Port of `ocr::reset_input` (`native_osc_macos/ocr.rs:33`): the next press
/// starts a fresh selection instead of a text interaction.
pub(crate) fn reset_text_recognition_input(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    if context.runtime.purpose != Purpose::TextRecognition {
      return false;
    }
    context
      .runtime
      .completed
      .store(false, std::sync::atomic::Ordering::Release);
    if let Ok(mut controller) = context.runtime.controller.lock() {
      let _ = controller.set_committed(None);
    }
    true
  })
  .unwrap_or(false)
}

pub(crate) fn set_snapshot_presented(window: &WebviewWindow, presented: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.snapshot.presented = presented;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.snapshot_presented = presented;
        surface.draw();
      }
    }
  })
  .is_some()
}

/// Port of the eight `native_osc_ruler_*` pulls (`native_osc_macos/state.rs:48-285`).
/// macOS needed a count-then-fill C convention; here the document hands back
/// vectors, so one pass produces the whole draw set.
fn pull_ruler(context: &Context) -> Option<RulerData> {
  if context.runtime.purpose != Purpose::Ruler {
    return None;
  }
  let app = context.runtime.window.app_handle();
  let state = app.state::<crate::ruler::RulerState>();
  let (centerlines, inner_objects) = state.center_aids();
  Some(RulerData {
    measurements: state.measurements().iter().map(Into::into).collect(),
    viewports: state.viewports().iter().map(Into::into).collect(),
    probes: state.probes().iter().map(Into::into).collect(),
    guides: state.guides().iter().map(Into::into).collect(),
    guide_gaps: state.guide_gaps().iter().map(Into::into).collect(),
    radii: state.radii().iter().map(Into::into).collect(),
    centerlines: centerlines.iter().map(Into::into).collect(),
    inner_objects: inner_objects.iter().map(Into::into).collect(),
  })
}

/// Port of `screenwide_region_osc_apply_ruler_result` (`+ruler.m:919-1071`):
/// pull every dataset, mirror the flags onto each surface, diff, re-assign the
/// label pools and redraw.
pub(crate) fn apply_ruler_result(context: &Context, result: &OscResult) -> bool {
  if result.ruler_flags & 1 == 0 {
    return false;
  }
  let Some(data) = pull_ruler(context) else {
    return false;
  };
  let now = Instant::now();
  let crosshair = result.ruler_flags & 2 != 0;
  let copied = result.ruler_flags & 4 != 0;
  let interaction_active = result.ruler_flags & 64 != 0;
  let tolerance_requested = result.ruler_flags & 8 != 0;
  let tolerance_mode = (result.ruler_flags >> 4) & 3;
  let (hover_key, hover_opacity) = ruler::hovered_artifact_key(&data);
  let animating = ruler::animation_active(&data, result.ruler_flags);
  // Every peer gets the same union size, so the loupe keeps one width while
  // the pointer crosses monitors (`reserved_dimensions_length`).
  let desktop_size = context
    .runtime
    .desktop
    .lock()
    .ok()
    .and_then(|binding| binding.as_ref().map(|binding| binding.size))
    .unwrap_or_default();

  let Ok(mut set) = context.surfaces.lock() else {
    return false;
  };
  let root = set.root_mut();
  let tolerance_started = tolerance_requested
    && (!root.ruler.tolerance_visible || root.ruler.tolerance_mode != tolerance_mode);
  let hover_changed = root.ruler.hovered_artifact_key() != hover_key;
  let hover_started = if hover_changed {
    now
  } else {
    root.ruler.hover_started()
  };
  let hwnd = set.root_hwnd();

  let mut world = Vec::new();
  for surface in set.all_mut() {
    let viewport = data
      .viewports
      .iter()
      .find(|viewport| viewport.display_id == surface.display_id);
    let zoom = viewport.map_or(1.0, |viewport| viewport.zoom);
    let origin = viewport.map_or_else(Point::default, |viewport| Point {
      x: viewport.origin_x,
      y: viewport.origin_y,
    });
    let viewport_changed =
      surface.ruler.viewport_zoom != zoom || surface.ruler.viewport_origin != origin;
    surface.ruler.viewport_zoom = zoom;
    surface.ruler.viewport_origin = origin;
    surface.ruler.visible = true;
    surface.ruler.crosshair = crosshair;
    surface.ruler.interaction_active = interaction_active;
    surface.ruler.color = result.ruler_color;
    surface.ruler.desktop_size = desktop_size;
    let offset = surface.desktop_offset();
    surface.ruler.point = Point {
      x: result.x - offset.x,
      y: result.y - offset.y,
    };
    if tolerance_requested {
      surface.ruler.tolerance_mode = tolerance_mode;
    }
    surface.ruler.tolerance_visible = tolerance_requested;
    surface
      .ruler
      .set_tolerance(tolerance_requested, tolerance_started, now);
    surface
      .ruler
      .set_hover(hover_key, hover_opacity, hover_started);
    surface.ruler.replace_data(&data, viewport_changed);
    surface.ruler.set_copied(copied, now);
    let bounds = surface.logical_size();
    world.push(surface.ruler.visible_world_rect(offset, bounds));
  }
  // Which display owns a label can only be answered with every viewport in
  // hand, so the four pools are assigned here rather than per surface.
  let owned = ruler::assign_labels(&world, &data);
  for (index, surface) in set.all_mut().enumerate() {
    surface
      .ruler
      .set_labels(owned.get(index).cloned().unwrap_or_default());
    // The loupe follows the pointer inside the one swap chain, so every ruler
    // result redraws; macOS could skip the world pass because its readout was
    // a separate layer.
    surface.draw();
  }
  drop(set);

  if animating {
    let _ = unsafe { SetTimer(Some(hwnd), input::RULER_SETTLE_TIMER, 16, None) };
  }
  if copied {
    let _ = unsafe {
      SetTimer(
        Some(hwnd),
        input::RULER_COPIED_TIMER,
        ruler::EXPIRY.as_millis() as u32,
        None,
      )
    };
  }
  if tolerance_started {
    let _ = unsafe {
      SetTimer(
        Some(hwnd),
        input::RULER_TOLERANCE_TIMER,
        ruler::EXPIRY.as_millis() as u32,
        None,
      )
    };
  }
  true
}

/// The 16ms settle frame macOS scheduled with `dispatch_after`
/// (`schedule_settle_frame`, `+ruler.m:836-849`).
pub(crate) fn ruler_settle_frame(context: &Context) {
  let live = with_set(context, |set| {
    set.root_mut().input_enabled && set.root_mut().visible
  })
  .unwrap_or(false);
  if !live {
    return;
  }
  let result = dispatch_input(context, 15, Point::default(), 0);
  if result.status != 255 {
    apply_ruler_result(context, &result);
  }
}

/// The copied checkmark's 900ms expiry (`+ruler.m:1058-1070`).
pub(crate) fn ruler_expire_copied(context: &Context) {
  with_set(context, |set| {
    let now = Instant::now();
    for surface in set.all_mut() {
      surface.ruler.set_copied(false, now);
      surface.draw();
    }
  });
}

/// The tolerance notice's 900ms expiry (`+ruler.m:1044-1057`).
pub(crate) fn ruler_expire_tolerance(context: &Context) {
  with_set(context, |set| {
    let now = Instant::now();
    for surface in set.all_mut() {
      surface.ruler.tolerance_visible = false;
      surface.ruler.set_tolerance(false, false, now);
      surface.draw();
    }
  });
}

fn with_set<T>(context: &Context, work: impl FnOnce(&mut SurfaceSet) -> T) -> Option<T> {
  context.surfaces.lock().ok().map(|mut set| work(&mut set))
}

/// Port of `screenwide_region_osc_ruler_refresh_pointer` (`+input.m:354-379`):
/// re-samples the pointer so the readout resumes without waiting for a move.
pub(crate) fn refresh_ruler_pointer(window: &WebviewWindow) -> bool {
  let Some(context) = context_arc(window) else {
    return false;
  };
  let Some(pointer) = surface::cursor_position() else {
    return false;
  };
  let target = with_set(&context, |set| {
    set
      .all_mut()
      .find(|surface| {
        surface.input_enabled && surface.visible && surface.contains_screen_point(pointer)
      })
      .map(|surface| {
        let (x, y) = surface.screen_to_client(pointer.x, pointer.y);
        surface.desktop_point(surface.logical_point(x, y))
      })
  })
  .flatten();
  let Some(point) = target else {
    return false;
  };
  let result = dispatch_input(&context, 1, point, 0);
  if result.status == 255 {
    return false;
  }
  apply_ruler_result(&context, &result)
}

/// Port of `screenwide_region_osc_ruler_set_transient_chrome`
/// (`+ruler.m:1121-1143`): hides the loupe and the live probes while a
/// screenshot is being taken through the frozen desktop.
pub(crate) fn set_ruler_transient_chrome(window: &WebviewWindow, visible: bool) -> bool {
  with_surfaces(window, |set| {
    for surface in set.all_mut() {
      surface.ruler.transient_chrome = visible;
      surface.draw();
    }
  })
  .is_some()
}

pub(crate) fn set_snapshot_composited(window: &WebviewWindow, composited: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.snapshot.composited = composited;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.snapshot_composited = composited;
        surface.draw();
      }
    }
  })
  .is_some()
}
#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
