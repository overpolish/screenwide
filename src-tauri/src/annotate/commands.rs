// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotate tools on the island ask for: the same three steps their
//! keys take, and the keyboard back.
//!
//! Every step is the keyboard's own path, so a button and its shortcut
//! cannot drift apart.

use tauri::AppHandle;

/// Takes the newest annotation off the screen, as Cmd+Z (Ctrl+Z on Windows)
/// over the overlay does. A running recording keeps the clip it earned: the
/// annotation really was visible for that long.
#[tauri::command]
pub fn undo_annotation(app: AppHandle) {
  if !super::is_active(&app) {
    return;
  }
  if super::live_clips::remove_last() {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    super::native_overlay::redraw();
  }
}

/// Takes every annotation off the screen, as `Backspace` over the overlay does.
#[tauri::command]
pub fn clear_annotations(app: AppHandle) {
  super::clear(&app);
}

/// Leaves the overlay, as Escape and the shortcut do. Annotations asked to stay
/// stay; the rest go with it.
#[tauri::command]
pub fn dismiss_annotate(app: AppHandle) {
  super::dismiss(&app);
}

/// A press on the island is done with. The keyboard goes back to the
/// picture, where the letters pick tools up again.
#[tauri::command]
pub fn return_annotate_keyboard(app: AppHandle) {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if super::is_active(&app) {
    super::island::return_keyboard(&app);
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = app;
}
