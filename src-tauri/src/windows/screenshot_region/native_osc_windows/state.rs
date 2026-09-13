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

#[path = "state/attachment.rs"]
mod attachment;
#[path = "state/content.rs"]
mod content;
#[path = "state/controls.rs"]
mod controls;
#[path = "state/desktop.rs"]
mod desktop;
#[path = "state/ruler_updates.rs"]
mod ruler_updates;
#[path = "state/scene.rs"]
mod scene;
use attachment::{context_arc, with_surfaces};
pub(crate) use attachment::{
  ensure_attached, ensure_ruler_attached, ensure_text_recognition_attached, focus_ruler_input,
  input_hwnd, set_capture_affinity, with_context,
};
pub(crate) use content::{
  reset_text_recognition_input, set_ocr, set_ocr_cancel_visible, set_snapshot,
  set_snapshot_composited, set_snapshot_presented,
};
use controls::set_exclusion_rect;
pub(crate) use controls::{
  claim_pointer_surface, set_allow_drawing, set_aspect, set_desktop_presented, set_input_enabled,
  set_magnifier_source, set_monitor, set_show_frame, set_show_handles,
};

pub(crate) use desktop::{
  configure_desktop, configure_desktop_window, notify_layout_changed_for_surface,
};
pub(crate) use ruler_updates::{
  apply_ruler_result, refresh_ruler_pointer, ruler_expire_copied, ruler_expire_tolerance,
  ruler_settle_frame, set_ruler_transient_chrome,
};

pub(crate) use scene::{
  apply_region_scene, clear_region, present_region, reconcile_region_scene_request, region_scene,
  region_scene_request_base, restore_normal_region_scene, set_committed,
};

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
  /// never notifies - AppKit only fired the callback from its screen-parameters
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

/// Port of `screenwide_region_osc_ocr_set_cancel_visible` (`+ocr_cancel.m:166`).
#[expect(
  dead_code,
  reason = "the reusable cancel OSC is intentionally not shown by OCR"
)]
#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;

#[cfg(test)]
use controls::gesture_after_input_update;
#[cfg(test)]
pub(crate) use desktop::layout_changed;
