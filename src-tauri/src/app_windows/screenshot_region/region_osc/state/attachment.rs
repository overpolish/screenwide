// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn host(window: &WebviewWindow) -> Option<WindowId> {
  super::super::host::host().webview_window(window)
}

pub(super) fn slot(hwnd: WindowId) -> Option<ContextSlot> {
  let mut map = contexts().lock().ok()?;
  Some(Arc::clone(
    map
      .entry(hwnd.0)
      .or_insert_with(|| Arc::new(OnceLock::new())),
  ))
}

pub(super) fn attach(window: &WebviewWindow, width: f64, height: f64, purpose: Purpose) -> bool {
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

pub(super) fn create(
  window: &WebviewWindow,
  hwnd: WindowId,
  width: f64,
  height: f64,
  purpose: Purpose,
) -> Result<Arc<Context>, String> {
  let overlay = super::super::host::host()
    .create_window(window, hwnd, None)
    .inspect_err(|error| {
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
      backdrops: HashMap::new(),
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
/// context.
pub(crate) fn ensure_text_recognition_attached(
  window: &WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(window, |_| ()).is_some() || attach(window, width, height, Purpose::TextRecognition)
}

/// The Ruler twin: same idempotent attach, `Purpose::Ruler` context
///.
pub(crate) fn ensure_ruler_attached(window: &WebviewWindow, width: f64, height: f64) -> bool {
  with_context(window, |_| ()).is_some() || attach(window, width, height, Purpose::Ruler)
}

pub(crate) fn input_hwnd(window: &WebviewWindow) -> Option<isize> {
  with_context(window, |context| {
    context.surfaces.lock().ok().map(|set| set.root_hwnd().0)
  })
  .flatten()
}

/// Routes the keyboard to `window`'s overlay for `owner` until released.
pub(crate) fn capture_keyboard(
  window: &WebviewWindow,
  owner: crate::app_windows::screenshot_region::region_osc::host::KeyboardOwner,
) -> Result<(), String> {
  let target =
    input_hwnd(window).ok_or_else(|| "Could not locate the overlay's input surface".to_owned())?;
  crate::app_windows::screenshot_region::region_osc::host::host().capture_keyboard(
    crate::app_windows::screenshot_region::region_osc::host::WindowId(target),
    owner,
  )
}

pub(crate) fn release_keyboard(
  owner: crate::app_windows::screenshot_region::region_osc::host::KeyboardOwner,
) {
  crate::app_windows::screenshot_region::region_osc::host::host().release_keyboard(owner);
}

/// Whether the caller is on the thread that owns `window`'s views.
pub(crate) fn on_owning_thread(window: &WebviewWindow) -> bool {
  crate::app_windows::screenshot_region::region_osc::host::host().on_owning_thread(window)
}

/// Gives the nonactivating compositor child keyboard focus after its Tauri
/// host has become foreground. This mirrors macOS making the Ruler panel key
/// and also provides a direct WM_KEYDOWN path alongside the low-level hook.
pub(crate) fn focus_ruler_input(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    let hwnd = context.surfaces.lock().ok().map(|set| set.root_hwnd());
    hwnd.is_some_and(|hwnd| super::super::host::host().focus(hwnd))
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
    Arc::clone(map.get(&hwnd.0)?)
  };
  let context = slot.get()?.as_ref().ok()?;
  Some(work(context))
}

/// Runs `work` over the whole surface set; `None` when unattached.
pub(super) fn with_surfaces<T>(
  window: &WebviewWindow,
  work: impl FnOnce(&mut SurfaceSet) -> T,
) -> Option<T> {
  with_context(window, |context| {
    context.surfaces.lock().ok().map(|mut set| work(&mut set))
  })
  .flatten()
}

/// Updates the top-level desktop peers alongside the host webview's affinity.
/// The root overlay is part of the webview's window and inherits its capture
/// treatment.
pub(crate) fn set_capture_affinity(window: &WebviewWindow, capturable: bool) -> bool {
  with_surfaces(window, |set| {
    set
      .peers
      .iter()
      .all(|peer| super::super::host::host().set_capture_affinity(peer.hwnd(), capturable))
  })
  .unwrap_or(true)
}

pub(super) fn context_arc(window: &WebviewWindow) -> Option<Arc<Context>> {
  let hwnd = host(window)?;
  let slot = {
    let map = contexts().lock().ok()?;
    Arc::clone(map.get(&hwnd.0)?)
  };
  slot.get()?.as_ref().ok().map(Arc::clone)
}
