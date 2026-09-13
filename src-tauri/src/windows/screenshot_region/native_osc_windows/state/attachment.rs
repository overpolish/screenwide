// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn host(window: &WebviewWindow) -> Option<HWND> {
  // Tauri hands back its own `windows` binding's handle; the value is the
  // same process-wide token.
  window.hwnd().ok().map(|hwnd| HWND(hwnd.0))
}

pub(super) fn slot(hwnd: HWND) -> Option<ContextSlot> {
  let mut map = contexts().lock().ok()?;
  Some(Arc::clone(
    map
      .entry(hwnd.0 as isize)
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

pub(super) fn context_arc(window: &WebviewWindow) -> Option<Arc<Context>> {
  let hwnd = host(window)?;
  let slot = {
    let map = contexts().lock().ok()?;
    Arc::clone(map.get(&(hwnd.0 as isize))?)
  };
  slot.get()?.as_ref().ok().map(Arc::clone)
}
