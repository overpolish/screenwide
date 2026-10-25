// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the island meets each platform while the Ruler is up: how it stands
//! above the overlay, keeps the pointer an arrow, and stays out of the
//! desktop the Ruler freezes.

use tauri::AppHandle;

use crate::app_windows;

/// Windows has no window levels, and a press on the overlay raises it over
/// every other topmost window, so the island stands above it by being owned
/// by the Ruler's window: an owned window always stands above its owner.
#[cfg(target_os = "windows")]
pub(super) fn own_by_ruler(app: &AppHandle, owned: bool) {
  let Some(island) = app_windows::island::window(app) else {
    return;
  };
  let owner = if owned {
    super::super::ruler_windows(app).into_iter().next()
  } else {
    None
  };
  if owned && owner.is_none() {
    return;
  }
  if let Err(error) = app_windows::platform::set_owner(&island, owner.as_ref()) {
    eprintln!("Could not set the island's owner: {error}");
  }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
  /// `src/ruler/cursor_guard_macos.m`: the window over which the overlay
  /// leaves the pointer alone, or none.
  fn screenwide_set_cursor_guard_exempt_window(window: *mut std::ffi::c_void);
}

/// The island is chrome rather than desktop, so over it the pointer is the
/// arrow its controls are pressed with, not the Ruler's crosshair. The
/// overlay is the key window and is sent every pointer move, wherever the
/// pointer is, so it is told which window to leave the pointer to.
#[cfg(target_os = "macos")]
pub(super) fn exempt_from_crosshair(app: &AppHandle, exempt: bool) {
  let island = exempt.then(|| app_windows::island::window(app)).flatten();
  let result = app.run_on_main_thread(move || {
    let window = island
      .as_ref()
      .and_then(|island| island.ns_window().ok())
      .unwrap_or(std::ptr::null_mut());
    unsafe { screenwide_set_cursor_guard_exempt_window(window) };
  });
  if let Err(error) = result {
    eprintln!("Could not reach the main thread to exempt the island: {error}");
  }
}

/// Keeps the island out of captures while the Ruler freezes the desktop, or
/// lets them see it again. The island stays on screen and changes into the
/// Ruler's tools over the frozen desktop; frozen into it as well, its old
/// shape would show round the new one.
pub(in crate::ruler) async fn keep_out_of_captures(app: &AppHandle, kept_out: bool) {
  let Some(island) = app_windows::island::window(app) else {
    return;
  };
  #[cfg(target_os = "macos")]
  {
    // The island is left visible to captures on macOS, where a recording
    // keeps it out by owning process instead.
    if let Err(error) = app_windows::set_window_capture_affinity(&island, !kept_out) {
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
      app_windows::set_window_capture_affinity(&island, false)
    } else {
      app_windows::apply_capture_policy(app)
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
  let _ = (island, kept_out);
}
