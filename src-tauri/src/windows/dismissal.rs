// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::WebviewWindow;

/// Hide an ordinary window without promoting another Screenwide window.
pub fn hide_without_focus_transfer(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "macos")]
  {
    fn hide(window: &WebviewWindow) -> tauri::Result<()> {
      unsafe extern "C" {
        fn screenwide_dismiss_window(
          window: *mut std::ffi::c_void,
          context: *mut std::ffi::c_void,
          completion: extern "C" fn(*mut std::ffi::c_void, bool),
        );
      }
      extern "C" fn completed(context: *mut std::ffi::c_void, hidden: bool) {
        let window = unsafe { Box::from_raw(context.cast::<WebviewWindow>()) };
        if hidden {
          use tauri::Manager;
          let _ = super::sync_dock_visibility(window.app_handle());
        }
      }
      let raw_window = window.ns_window()?;
      let context = Box::into_raw(Box::new(window.clone())).cast();
      unsafe { screenwide_dismiss_window(raw_window, context, completed) };
      Ok(())
    }

    if objc2::MainThreadMarker::new().is_some() {
      return hide(window);
    }
    let native_window = window.clone();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    window.run_on_main_thread(move || {
      let _ = sender.send(hide(&native_window));
    })?;
    receiver.recv().map_err(std::io::Error::other)?
  }
  #[cfg(not(target_os = "macos"))]
  window.hide()
}

/// Cancel a delayed hide when the user explicitly reopens the window.
#[cfg(target_os = "macos")]
pub fn cancel_pending_dismissal(window: &WebviewWindow) -> tauri::Result<()> {
  let window = window.clone();
  window.clone().run_on_main_thread(move || {
    unsafe extern "C" {
      fn screenwide_cancel_pending_dismissal(window: *mut std::ffi::c_void);
    }
    if let Ok(raw_window) = window.ns_window() {
      unsafe { screenwide_cancel_pending_dismissal(raw_window) };
    }
  })
}
