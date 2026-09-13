// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub fn set_recording_preview_zoom(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  zoom_percent: f64,
) -> Result<(), String> {
  if !zoom_percent.is_finite() || !(10.0..=1_600.0).contains(&zoom_percent) {
    return Err("The recording preview zoom is invalid".to_owned());
  }
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
    surface.set_editor_zoom(zoom_percent);
  }
  Ok(())
}

/// Applies a one-time native fit after the editor makes room for a panel.
/// Omit fit_width for the ordinary full-viewport reset.
#[tauri::command]
pub fn reset_recording_preview_view(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  fit_width: Option<f64>,
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
    surface.reset_editor_view(fit_width);
  }
  Ok(())
}

/// Moves the basis a double-click reset returns to without moving the view:
/// a tool panel that reserved space hands the basis back on closing, leaving
/// the picture exactly where the user left it.
#[tauri::command]
pub fn set_recording_preview_fit_basis(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  fit_width: Option<f64>,
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
    surface.set_editor_fit_basis(fit_width);
  }
  Ok(())
}
