// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editor suspension for the recording preview surface.

use super::RecordingPreviewPlayerState;

/// Takes the native editor's input and chrome off screen while the frontend
/// covers the workarea (a save in progress, the export window). The workspace
/// transform is preserved, so this is not a disable: resuming needs no pan or
/// zoom restore.
#[tauri::command]
pub fn set_recording_preview_editor_suspended(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  suspended: bool,
) -> Result<(), String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  if let Some(surface) = manager
    .sources
    .as_ref()
    .and_then(|sources| sources.preview_surface.as_ref())
  {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    surface.set_editor_suspended(suspended);
  }
  Ok(())
}

/// Puts the recording editor's view back at 100%, centred: ⌘0 (Ctrl+0).
pub(crate) fn reset_view(state: &RecordingPreviewPlayerState) -> Result<(), String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  if let Some(surface) = manager
    .sources
    .as_ref()
    .and_then(|sources| sources.preview_surface.as_ref())
  {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    surface.reset_view();
  }
  Ok(())
}

/// Hands the keyboard back to a box or caption being typed into in the
/// recording editor.
pub(crate) fn resume_typing(state: &RecordingPreviewPlayerState) -> Result<(), String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  if let Some(surface) = manager
    .sources
    .as_ref()
    .and_then(|sources| sources.preview_surface.as_ref())
  {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    surface.resume_annotation_text();
  }
  Ok(())
}
