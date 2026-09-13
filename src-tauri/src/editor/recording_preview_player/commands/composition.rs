// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

// Async for the same reason as `layout_recording_preview_surface`: a paused
// redraw must not block the main thread's input pump.
#[tauri::command]
pub async fn set_recording_preview_composition(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  bake_camera: bool,
  camera_overlay: CameraOverlaySettings,
  recording_output: RecordingOutputSettings,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let settings = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?
    .composition_settings
    .clone()
    .ok_or_else(|| "The recording preview composition is unavailable".to_owned())?;
  let next = PreviewCompositionSettings {
    bake_camera,
    camera_overlay,
    recording_output: recording_output.clone(),
  };
  let current = settings
    .read()
    .map_err(|_| "The recording preview composition is unavailable".to_owned())?;
  if *current == next {
    return Ok(());
  }
  #[cfg(target_os = "windows")]
  let bake_changed = current.bake_camera != bake_camera;
  drop(current);
  *settings
    .write()
    .map_err(|_| "The recording preview composition is unavailable".to_owned())? = next;
  if !manager.is_playing {
    // A composition change leaves the decoded frame and its cursor valid, so
    // Windows redraws the paused still synchronously from the cached sources
    // instead of restarting the decoder; a full reopen per pointer move made
    // camera drags trail the pointer. A bake toggle keeps the decoder path -
    // the newly active mode's source cache is absent or stale - and so do
    // cursor-effect changes, whose composition is stale by definition.
    #[cfg(target_os = "windows")]
    if !bake_changed {
      let has_camera = manager
        .sources
        .as_ref()
        .is_some_and(|sources| sources.camera_path.is_some());
      let redrawn = manager
        .sources
        .as_ref()
        .and_then(|sources| sources.preview_surface.as_ref())
        .is_some_and(|surface| {
          surface
            .redraw_still(
              bake_camera && has_camera,
              &recording_output.primary,
              &recording_output.camera,
              camera_overlay,
              recording_output.camera.drop_shadow,
              recording_output.camera_on_top,
            )
            .unwrap_or(false)
        });
      if redrawn {
        return Ok(());
      }
    }
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}

#[tauri::command]
pub fn set_recording_preview_cursor_effects(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  cursor_effects: CursorEffectSettings,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let settings = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?
    .cursor_settings
    .clone();
  *settings
    .write()
    .map_err(|_| "The cursor preview settings are unavailable".to_owned())? = cursor_effects;
  if !manager.is_playing {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}
