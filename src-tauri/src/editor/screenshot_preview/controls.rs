// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::state::ScreenshotPreviewState;

#[tauri::command]
pub fn stop_screenshot_preview(
  state: tauri::State<'_, ScreenshotPreviewState>,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
  if manager.session_id == Some(session_id) {
    manager.stop();
  }
  Ok(())
}

/// Stops the screenshot preview whatever its session, for an editor window
/// about to be destroyed.
pub(crate) fn stop_all(app: &tauri::AppHandle) {
  use tauri::Manager;
  if let Some(state) = app.try_state::<ScreenshotPreviewState>() {
    if let Ok(mut manager) = state.0.lock() {
      manager.stop();
    }
  }
}

/// Screenshot counterpart of `set_recording_preview_editor_suspended`: input
/// and native chrome off, workspace transform kept.
#[tauri::command]
pub fn set_screenshot_preview_editor_suspended(
  state: tauri::State<'_, ScreenshotPreviewState>,
  session_id: u64,
  suspended: bool,
) -> Result<(), String> {
  let surface = {
    let manager = state
      .0
      .lock()
      .map_err(|_| "The screenshot preview is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    manager.surface.clone()
  };
  if let Some(surface) = surface {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    surface.set_editor_suspended(suspended);
  }
  Ok(())
}
