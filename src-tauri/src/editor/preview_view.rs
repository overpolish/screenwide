// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! ⌘0 (Ctrl+0) in an editor window: its picture's view back at 100%,
//! centred, as a double-click on empty space puts it.

use tauri::Manager;

use super::workspace_kind::EditorKind;

/// Resets the view of the editor `window` is.
#[tauri::command]
pub fn reset_editor_view(window: tauri::WebviewWindow) -> Result<(), String> {
  let app = window.app_handle();
  match EditorKind::from_window_label(window.label()) {
    Some(EditorKind::Recording) => super::recording_preview_player::editor_suspend::reset_view(
      &app.state::<super::recording_preview_player::RecordingPreviewPlayerState>(),
    ),
    Some(EditorKind::Screenshot) => super::screenshot_preview::reset_view(
      &app.state::<super::screenshot_preview::ScreenshotPreviewState>(),
    ),
    None => Ok(()),
  }
}
