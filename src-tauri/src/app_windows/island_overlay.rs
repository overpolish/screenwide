// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island over a capture overlay that shows its tools there, the Ruler
//! or Text Recognition: how it stands above the overlay, keeps the pointer
//! an arrow, and stays out of the desktop the overlay freezes.

use tauri::{AppHandle, WebviewWindow};

use super::island;

/// Stands the island over `overlay`, the window the tool draws in, or lets
/// it go back to its own level with none.
///
/// Windows has no window levels, and a press on the overlay raises it over
/// every other topmost window, so there the island stands above it by being
/// owned by it: an owned window always stands above its owner. Call it once
/// the overlay is presented, and with none before the overlay closes, as a
/// window takes the windows it owns with it when it is destroyed.
pub(crate) fn stand_over(app: &AppHandle, overlay: Option<&WebviewWindow>) {
  if let Err(error) = island::set_above_overlays(app, overlay.is_some()) {
    eprintln!("Could not change the island's level: {error}");
  }
  #[cfg(target_os = "windows")]
  if let Some(window) = island::window(app) {
    if let Err(error) = super::platform::set_owner(&window, overlay) {
      eprintln!("Could not set the island's owner: {error}");
    }
  }
  #[cfg(target_os = "macos")]
  exempt_from_crosshair(app, overlay.is_some());
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
  /// `src/ruler/cursor_guard_macos.m`: the window over which the overlay
  /// leaves the pointer alone, or none.
  fn screenwide_set_cursor_guard_exempt_window(window: *mut std::ffi::c_void);
}

/// The island is chrome rather than desktop, so over it the pointer is the
/// arrow its controls are pressed with, not the overlay's crosshair. The
/// overlay is the key window and is sent every pointer move, wherever the
/// pointer is, so it is told which window to leave the pointer to.
#[cfg(target_os = "macos")]
fn exempt_from_crosshair(app: &AppHandle, exempt: bool) {
  let window = exempt.then(|| island::window(app)).flatten();
  let result = app.run_on_main_thread(move || {
    let native = window
      .as_ref()
      .and_then(|window| window.ns_window().ok())
      .unwrap_or(std::ptr::null_mut());
    unsafe { screenwide_set_cursor_guard_exempt_window(native) };
  });
  if let Err(error) = result {
    eprintln!("Could not reach the main thread to exempt the island: {error}");
  }
}

/// Keeps the island out of captures while a tool freezes the desktop, or
/// lets them see it again. The island stays on screen and changes into the
/// tool's over the frozen desktop; frozen into it as well, its old shape
/// would show round the new one.
pub(crate) async fn keep_out_of_captures(app: &AppHandle, kept_out: bool) {
  let Some(window) = island::window(app) else {
    return;
  };
  #[cfg(target_os = "macos")]
  {
    // The island is left visible to captures on macOS, where a recording
    // keeps it out by owning process instead.
    if let Err(error) = super::set_window_capture_affinity(&window, !kept_out) {
      eprintln!("Could not change the island's capture affinity: {error}");
      return;
    }
    // The change is made on the main thread's next turn; wait for it, so the
    // capture that follows cannot run ahead of it.
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    if app
      .run_on_main_thread(move || {
        let _ = sender.send(());
      })
      .is_ok()
    {
      let _ = receiver.recv();
    }
  }
  #[cfg(target_os = "windows")]
  {
    let result = if kept_out {
      super::set_window_capture_affinity(&window, false)
    } else {
      super::apply_capture_policy(app)
    };
    if let Err(error) = result {
      eprintln!("Could not change the island's capture affinity: {error}");
      return;
    }
    if kept_out {
      // Display affinity is committed by DWM. Let that frame land before
      // the capture, as the Ruler's screenshot mode does.
      tokio::time::sleep(std::time::Duration::from_millis(75)).await;
    }
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = (window, kept_out);
}
