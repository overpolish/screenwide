// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The webview handing the native preview its scene clips.

use super::*;
use screenwide_scenes::{validate_clips, validate_held_clip, GapScene, RecordingSceneClip};

/// Hands the preview the recording's scene clips, the gaps drawn among them
/// and the timeline they are timed on, as `playback_ranges` lay it out,
/// redrawing a paused frame that they change. Where a scene follows the face
/// before it has been found in the camera, finding it starts.
#[tauri::command]
pub async fn set_recording_preview_scenes(
  app: tauri::AppHandle,
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingSceneClip>,
  gaps: Vec<GapScene>,
  playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  held: Option<RecordingSceneClip>,
) -> Result<(), String> {
  validate_clips(&clips)?;
  if let Some(held) = &held {
    validate_held_clip(held)?;
  }
  let ranges = super::animation_timeline_ranges(&playback_ranges);
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let sources = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?;
  let camera_follows = sources
    .composition_settings
    .as_ref()
    .and_then(|settings| {
      settings
        .read()
        .ok()
        .map(|settings| settings.camera_overlay.follow_face)
    })
    .unwrap_or(false);
  let unfound =
    sources.awaits_face(camera_follows || clips.iter().any(|clip| clip.follow_face == Some(true)));
  let mut current = sources
    .scenes
    .clips
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  let mut current_ranges = sources
    .scenes
    .ranges
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  // A drag on the picture sets the clips itself as it goes (`scene_gesture`),
  // and the webview's copy of the same drag lags it: drawing that copy would
  // pull the pane back a step each time one lands. Once the drag ends, the
  // webview's clips, which hold its last step, are drawn as ever.
  let dragging = manager.selection_gesture.is_some();
  let clips_changed = !dragging && current.set(clips, Some(gaps));
  let held_changed = current.hold(held);
  let changed = clips_changed || held_changed || *current_ranges != ranges;
  *current_ranges = ranges;
  drop(current);
  drop(current_ranges);
  let artifact_id = manager.artifact_id;
  let redrawn = if changed {
    manager.redraw_paused_frame()
  } else {
    Ok(())
  };
  drop(manager);
  // A project reopened with a scene that follows the face, but no face kept
  // beside it, finds it again as well as a scene just set to follow.
  super::face_preview::find_face_when(unfound, app, artifact_id);
  redrawn
}
