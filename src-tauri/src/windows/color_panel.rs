// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system Colours panel, opened by the web UI's colour wells.
//!
//! WebKit's `<input type="color">` opens a popover of its own that we can
//! neither place nor theme, so on macOS the well asks for `NSColorPanel`
//! instead. The panel is a single shared system object, so only the well that
//! asked last hears about changes: it passes a token that every event carries
//! back, and a well ignores anything that is not its own.

/// Emitted to the window that opened the panel whenever the colour changes,
/// which with a continuous panel is on every drag step.
#[cfg(target_os = "macos")]
const CHANGE_EVENT: &str = "color-panel-change";

/// Emitted to the window that opened the panel when the user closes it.
#[cfg(target_os = "macos")]
const CLOSE_EVENT: &str = "color-panel-close";

#[cfg(target_os = "macos")]
mod imp {
  use std::ffi::{c_char, c_void, CStr};
  use std::sync::{Mutex, OnceLock};

  use tauri::{AppHandle, Emitter, WebviewWindow};

  unsafe extern "C" {
    fn screenwide_color_panel_show(
      hex: *const c_char,
      on_change: extern "C" fn(*const c_char, *mut c_void),
      on_close: extern "C" fn(*mut c_void),
      context: *mut c_void,
    );
    fn screenwide_color_panel_close();
  }

  /// The well that owns the panel right now: the window to tell, and the
  /// token that window matches against. `None` once the panel has closed.
  static OWNER: Mutex<Option<(String, String)>> = Mutex::new(None);

  /// The handle the native callbacks emit through. Stored once so its address
  /// can serve as the panel's context pointer without leaking per call.
  static APP: OnceLock<AppHandle> = OnceLock::new();

  fn owner() -> Option<(String, String)> {
    OWNER.lock().ok().and_then(|owner| owner.clone())
  }

  /// # Safety
  /// `context` is the address of the `AppHandle` held by [`APP`], which lives
  /// for the process, so it is only read, never freed.
  fn app(context: *mut c_void) -> Option<&'static AppHandle> {
    unsafe { context.cast::<AppHandle>().as_ref() }
  }

  extern "C" fn on_change(hex: *const c_char, context: *mut c_void) {
    let (Some(app), Some((label, token))) = (app(context), owner()) else {
      return;
    };
    let Ok(color) = (unsafe { CStr::from_ptr(hex) }).to_str() else {
      return;
    };
    // Runs on the main thread, where AppKit sends the panel's action, so this
    // only ever queues the event rather than waiting on anything.
    let _ = app.emit_to(
      label,
      super::CHANGE_EVENT,
      serde_json::json!({ "token": token, "color": color }),
    );
  }

  extern "C" fn on_close(context: *mut c_void) {
    let (Some(app), Some((label, token))) = (app(context), owner()) else {
      return;
    };
    if let Ok(mut owner) = OWNER.lock() {
      *owner = None;
    }
    let _ = app.emit_to(
      label,
      super::CLOSE_EVENT,
      serde_json::json!({ "token": token }),
    );
  }

  pub fn show(window: &WebviewWindow, color: &str, token: String) -> Result<(), String> {
    use tauri::Manager;

    let app = APP.get_or_init(|| window.app_handle().clone());
    if let Ok(mut owner) = OWNER.lock() {
      *owner = Some((window.label().to_owned(), token));
    }
    let hex = std::ffi::CString::new(color).map_err(|error| error.to_string())?;
    let context = std::ptr::from_ref(app).cast_mut().cast::<c_void>();
    // Sync commands already run on the macOS main thread, which is where
    // AppKit requires this, so there is nothing to hop onto.
    unsafe { screenwide_color_panel_show(hex.as_ptr(), on_change, on_close, context) };
    Ok(())
  }

  pub fn close() -> Result<(), String> {
    if let Ok(mut owner) = OWNER.lock() {
      *owner = None;
    }
    unsafe { screenwide_color_panel_close() };
    Ok(())
  }
}

/// Opens the system Colours panel on `color`, and reports every change back to
/// the calling window tagged with `token`.
#[tauri::command]
pub fn show_color_panel(
  window: tauri::WebviewWindow,
  color: String,
  token: String,
) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  {
    imp::show(&window, &color, token)
  }
  #[cfg(not(target_os = "macos"))]
  {
    let _ = (window, color, token);
    Err("unsupported".to_owned())
  }
}

/// Closes the system Colours panel, without a close event: the caller asked.
#[tauri::command]
pub fn close_color_panel() -> Result<(), String> {
  #[cfg(target_os = "macos")]
  {
    imp::close()
  }
  #[cfg(not(target_os = "macos"))]
  {
    Err("unsupported".to_owned())
  }
}
