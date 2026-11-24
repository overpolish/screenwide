// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Screenwide shows a window while it is still a menu bar app: the Dock icon
//! that makes it an ordinary one arrives asynchronously. Until then macOS
//! opens the window on whatever Space is current, another app's full-screen
//! Space included, which the window then has no way to leave. A window
//! opened there goes to the desktop beside that Space instead, and the user
//! goes with it, as with any app that opens a window.

use tauri::WebviewWindow;

unsafe extern "C" {
  fn screenwide_desktop_beside_full_screen(window: u32) -> u64;
  fn screenwide_move_own_window_to_space(window: u32, space: u64) -> bool;
}

/// Called once `window` is shown.
pub(super) fn leave_full_screen_space(window: &WebviewWindow) -> tauri::Result<()> {
  let target = window.clone();
  window.run_on_main_thread(move || {
    let Ok(ns_window) = target.ns_window() else {
      return;
    };
    // SAFETY: Tauri hands out the live NSWindow, read here on the main thread.
    let number = unsafe { &*ns_window.cast::<objc2_app_kit::NSWindow>() }.windowNumber();
    let Ok(number) = u32::try_from(number) else {
      return;
    };
    let space = unsafe { screenwide_desktop_beside_full_screen(number) };
    if space == 0 || !unsafe { screenwide_move_own_window_to_space(number, space) } {
      return;
    }
    // Brought forward on its desktop, the window takes the user there.
    if let Err(error) = target.set_focus() {
      eprintln!("Could not bring the window forward on its desktop: {error}");
    }
  })
}
