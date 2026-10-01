// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows twin of `native_overlay_macos.rs`: the overlay's DirectComposition
//! surfaces.
//!
//! One surface per host window, all sharing one device, and each drawing its
//! display's annotations through the editor's own arrow shader. Nothing is
//! mirrored: every frame pulls [`super::live_clips`] and the stroke in hand,
//! so there is no second copy of the document to keep in step.
//!
//! macOS is handed a scene callback the compositor pulls through; here a
//! redraw does the pull itself, so there is no callback to install. Ordering a
//! host on screen is Tauri's `show`, not ours.
//!
//! Every surface belongs to the thread that owns its host window. Callers on
//! another thread reach it through [`redraw`], which dispatches; the rest are
//! documented as owning-thread only and are dispatched by `host.rs`.

mod bake;
pub(in super::super) use bake::bake;
mod frame;
mod renderer;
mod scene;
mod surface;
mod underlay;
mod window_proc;

use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock, RwLock};

use tauri::{AppHandle, Manager, WebviewWindow};
use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
      Input::KeyboardAndMouse::{
        GetCapture, GetFocus, GetKeyState, ReleaseCapture, SetCapture, SetFocus, VIRTUAL_KEY,
        VK_CONTROL, VK_SHIFT,
      },
      WindowsAndMessaging::{
        DefWindowProcW, DestroyWindow, GetClientRect, GetWindowLongPtrW, GetWindowThreadProcessId,
        LoadCursorW, SetCursor, SetWindowLongPtrW, SetWindowPos, ShowWindowAsync, GWLP_USERDATA,
        HTCLIENT, HTTRANSPARENT, HWND_TOP, IDC_CROSS, IDC_IBEAM, MA_NOACTIVATE, SWP_ASYNCWINDOWPOS,
        SWP_NOACTIVATE, SWP_NOOWNERZORDER, SW_SHOWNOACTIVATE, WM_CANCELMODE, WM_CAPTURECHANGED,
        WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
        WM_MOUSEACTIVATE, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCHITTEST,
        WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETCURSOR, WM_SYSKEYDOWN, WM_XBUTTONDOWN, WM_XBUTTONUP,
        WNDCLASS_STYLES,
      },
    },
  },
};

use super::super::{geometry, input, live_clips};

use crate::app_windows::overlay_surface;
use crate::editor::preview_platform::arrows;
use renderer::Renderer;
use surface::Surface;

/// One display the overlay covers: the capture display it is, where it
/// starts and how far it reaches in desktop points, and how many pixels one
/// point is.
#[derive(Clone, Copy)]
pub(in super::super) struct Display {
  pub id: u32,
  pub origin: (f64, f64),
  pub size: (f64, f64),
  pub scale: f64,
}

/// The surfaces on screen and the renderer they share. The renderer is made
/// by the first attach and dropped by the last detach: a session that is not
/// drawing holds no GPU resources.
#[derive(Default)]
struct Overlay {
  renderer: Option<Renderer>,
  surfaces: Vec<Surface>,
}

// The COM interfaces and window handles the overlay owns are process-wide
// tokens. Everything here is reached under this mutex, from the thread that
// owns the host windows.
unsafe impl Send for Overlay {}

static OVERLAY: LazyLock<Mutex<Overlay>> = LazyLock::new(|| Mutex::new(Overlay::default()));
static DISPLAYS: LazyLock<RwLock<Vec<Display>>> = LazyLock::new(|| RwLock::new(Vec::new()));
/// Kept so a redraw asked for off the owning thread can dispatch, and so the
/// key handler has a handle to write a setting with.
static APP: OnceLock<AppHandle> = OnceLock::new();

fn overlay() -> MutexGuard<'static, Overlay> {
  OVERLAY
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn display(index: u32) -> Option<Display> {
  DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .get(index as usize)
    .copied()
}

/// Names the displays the overlay is about to cover. Their order is the order
/// the host windows are created in, which is what `attach`'s index names.
pub(in super::super) fn set_displays(displays: Vec<Display>) {
  *DISPLAYS.write().unwrap_or_else(|error| error.into_inner()) = displays;
}

/// Forgets the displays, once nothing is drawn on them any more.
pub(in super::super) fn forget_displays() {
  set_displays(Vec::new());
  super::super::highlight::forget();
}

/// The display under a point in global desktop points, and its place in the
/// overlay's order.
pub(in super::super) fn display_at(x: f64, y: f64) -> Option<(usize, Display)> {
  DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .iter()
    .copied()
    .enumerate()
    .find(|(_, display)| {
      (display.origin.0..display.origin.0 + display.size.0).contains(&x)
        && (display.origin.1..display.origin.1 + display.size.1).contains(&y)
    })
}

/// Redraws every display from any thread.
pub(in super::super) fn request_redraw(_app: &AppHandle) {
  redraw();
}

/// Gives one host window its overlay child and surface. Owning thread only.
pub(in super::super) fn attach(window: &WebviewWindow, display: u32) -> Result<(), String> {
  let _ = APP.set(window.app_handle().clone());
  let host = host_window(window).ok_or_else(|| "The annotate host has no window".to_owned())?;
  let mut overlay = overlay();
  if overlay.surfaces.iter().any(|surface| surface.host == host) {
    return Ok(());
  }
  let renderer = match overlay.renderer.as_ref() {
    Some(renderer) => renderer,
    None => overlay.renderer.insert(Renderer::new()?),
  };
  let surface = Surface::new(renderer.gpu(), host, display)?;
  overlay.surfaces.push(surface);
  Ok(())
}

/// Takes one host window's overlay child away, and the renderer with it once
/// the last host is gone. Owning thread only.
pub(in super::super) fn detach(window: &WebviewWindow) {
  let Some(host) = host_window(window) else {
    return;
  };
  let mut overlay = overlay();
  overlay.surfaces.retain(|surface| surface.host != host);
  if overlay.surfaces.is_empty() {
    overlay.renderer = None;
  }
}

/// Gives the keyboard to the anchor's overlay child: the Windows twin of the
/// macOS anchor host being made key. The overlay hears keys only while it
/// holds focus. A press on the toolbar's web view moves focus into it without
/// activating anything, and neither side can get it back on its own, so every
/// way back to drawing (a press on the picture, a toolbar press that is done
/// with) ends here. Escape and the activation shortcut are global
/// registrations and work wherever focus is. Owning thread only.
pub(in super::super) fn focus_input() {
  let Some(child) = overlay().surfaces.first().map(|surface| surface.child) else {
    return;
  };
  if unsafe { GetFocus() } != child {
    let _ = unsafe { SetFocus(Some(child)) };
  }
}

/// Starts routing input: the keyboard goes to the anchor's overlay child.
pub(in super::super) fn install_input(app: &AppHandle) {
  let _ = APP.set(app.clone());
  focus_input();
}

/// Stops routing input. The surfaces stay: annotations left on screen are
/// still drawn on them.
///
/// A stroke in hand holds the pointer captured by its child. It is let go
/// here, before anything touches the surfaces: destroying a captured window
/// releases the capture itself, and the `WM_CAPTURECHANGED` that sends
/// redraws through the overlay's lock, which `detach` would be holding.
pub(in super::super) fn teardown_input() {
  let captured = unsafe { GetCapture() };
  let held = !captured.is_invalid()
    && overlay()
      .surfaces
      .iter()
      .any(|surface| surface.child == captured);
  if held {
    let _ = unsafe { ReleaseCapture() };
  }
}

/// Redraws every attached surface. Dispatches when it is called from a thread
/// that does not own the host windows, which is where the commands and the
/// shortcuts arrive from.
pub(in super::super) fn redraw() {
  if on_owning_thread() {
    frame::draw_all();
    return;
  }
  if let Some(app) = APP.get() {
    let _ = app.run_on_main_thread(frame::draw_all);
  }
}

/// Lets everything under a host through while its annotations stay drawn: the
/// display-only state annotations kept after exiting are shown in. The flag
/// lives on the child window so its procedure can read it without taking the
/// overlay's lock. Owning thread only.
pub(in super::super) fn set_click_through(window: &WebviewWindow, through: bool) {
  let Some(host) = host_window(window) else {
    return;
  };
  let overlay = overlay();
  if let Some(surface) = overlay.surfaces.iter().find(|surface| surface.host == host) {
    surface.set_click_through(through);
  }
}

/// The host's own window handle. Tauri hands back its own `HWND` newtype, so
/// the pointer is carried across rather than the type.
fn host_window(window: &WebviewWindow) -> Option<HWND> {
  window.hwnd().ok().map(|hwnd| HWND(hwnd.0))
}

/// True when this call is already on the thread that owns the host windows.
/// Blocking on `run_on_main_thread` from that thread would deadlock.
fn on_owning_thread() -> bool {
  let Some(host) = overlay().surfaces.first().map(|surface| surface.host) else {
    return false;
  };
  unsafe { GetWindowThreadProcessId(host, None) == GetCurrentThreadId() }
}
