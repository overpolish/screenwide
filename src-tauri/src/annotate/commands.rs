// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the overlay's toolbar asks for: the same three steps its keys take.
//!
//! Every one of them is the keyboard's own path, so a button and its shortcut
//! cannot drift apart.

use tauri::AppHandle;

/// Takes the newest annotation off the screen, as `Cmd+Z` over the overlay
/// does. A running recording keeps the clip it earned: the annotation really was
/// visible for that long.
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
