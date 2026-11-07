// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::WebviewWindow;

/// What runs once a window has been put away: `true` if it was, `false` if
/// it was shown again before it went.
pub(super) type AfterDismissal = Box<dyn FnOnce(&WebviewWindow, bool) + Send>;

/// Hide an ordinary window without promoting another Screenwide window.
pub fn hide_without_focus_transfer(window: &WebviewWindow) -> tauri::Result<()> {
  dismiss(window, None)
}

/// [`hide_without_focus_transfer`], then `after`. On macOS the window goes
/// once its dismissal finishes, which can be cancelled by showing it again.
pub(super) fn dismiss(window: &WebviewWindow, after: Option<AfterDismissal>) -> tauri::Result<()> {
  #[cfg(target_os = "macos")]
  {
    struct Dismissal {
      window: WebviewWindow,
      after: Option<AfterDismissal>,
    }

    fn hide(window: &WebviewWindow, after: Option<AfterDismissal>) -> tauri::Result<()> {
      unsafe extern "C" {
        fn screenwide_dismiss_window(
          window: *mut std::ffi::c_void,
          context: *mut std::ffi::c_void,
          completion: extern "C" fn(*mut std::ffi::c_void, bool),
        );
      }
      extern "C" fn completed(context: *mut std::ffi::c_void, hidden: bool) {
        let dismissal = unsafe { Box::from_raw(context.cast::<Dismissal>()) };
        let Dismissal { window, after } = *dismissal;
        if hidden {
          let _ = super::webview_visibility::hide_webview(&window);
          use tauri::Manager;
          let _ = super::dock_visibility::sync_after_dismissal(window.app_handle());
        }
        if let Some(after) = after {
          after(&window, hidden);
        }
      }
      let raw_window = window.ns_window()?;
      let context = Box::into_raw(Box::new(Dismissal {
        window: window.clone(),
        after,
      }))
      .cast();
      unsafe { screenwide_dismiss_window(raw_window, context, completed) };
      Ok(())
    }

    if objc2::MainThreadMarker::new().is_some() {
      return hide(window, after);
    }
    let native_window = window.clone();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    window.run_on_main_thread(move || {
      let _ = sender.send(hide(&native_window, after));
    })?;
    receiver.recv().map_err(std::io::Error::other)?
  }
  #[cfg(not(target_os = "macos"))]
  {
    window.hide()?;
    if let Some(after) = after {
      after(window, true);
    }
    Ok(())
  }
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
