// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::state::ScreenshotPreviewState;

#[tauri::command]
pub fn set_screenshot_preview_zoom(
  state: tauri::State<'_, ScreenshotPreviewState>,
  session_id: u64,
  zoom_percent: f64,
) -> Result<(), String> {
  if !zoom_percent.is_finite() || !(10.0..=1_600.0).contains(&zoom_percent) {
    return Err("The screenshot preview zoom is invalid".to_owned());
  }
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
    surface.set_editor_zoom(zoom_percent);
  }
  Ok(())
}

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

/// Screenshot counterpart of `reset_recording_preview_view`: zoom to fit, pan
/// reset, inside whatever the open tool panel leaves of the viewport.
#[tauri::command]
pub fn reset_screenshot_preview_view(
  state: tauri::State<'_, ScreenshotPreviewState>,
  session_id: u64,
  fit_width: Option<f64>,
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
    surface.reset_editor_view(fit_width);
  }
  Ok(())
}

/// Screenshot counterpart of `set_recording_preview_fit_basis`: moves the
/// basis a double-click reset returns to without moving the view.
#[tauri::command]
pub fn set_screenshot_preview_fit_basis(
  state: tauri::State<'_, ScreenshotPreviewState>,
  session_id: u64,
  fit_width: Option<f64>,
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
    surface.set_editor_fit_basis(fit_width);
  }
  Ok(())
}
