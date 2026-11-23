// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press in the island's panel window takes the keyboard from a box being
//! typed into in the editor beneath it; the island hands it back once the
//! press is done, so choosing a template or a colour keeps the typing going.

use tauri::{AppHandle, Manager};

use super::workspace_kind::EditorKind;

/// Hands the keyboard back to what is being typed into in the `workspace`
/// editor. Nothing happens where nothing is.
#[tauri::command]
pub fn resume_annotation_typing(app: AppHandle, workspace: EditorKind) -> Result<(), String> {
  match workspace {
    EditorKind::Recording => super::recording_preview_player::editor_suspend::resume_typing(
      &app.state::<super::recording_preview_player::RecordingPreviewPlayerState>(),
    ),
    EditorKind::Screenshot => super::screenshot_preview::resume_typing(
      &app.state::<super::screenshot_preview::ScreenshotPreviewState>(),
    ),
  }
}
