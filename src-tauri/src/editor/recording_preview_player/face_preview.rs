// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The face the preview follows: found when the camera or a scene first asks
//! to follow it, handed to the preview once it is there, and read by the
//! editor, which places a camera that follows it where the preview does.

use super::*;
use screenwide_scenes::{face::STEP_MS, FacePath};

/// Tells the editor the face path has changed, so it reads it again.
const FACE_PATH_EVENT: &str = "preview://face-path";

/// The face path as the editor reads it: its samples, `step_ms` apart from
/// the camera's start, each the face's centre as shares of the picture.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacePathView {
  step_ms: f64,
  points: Vec<[f64; 2]>,
}

/// The face path of the recording open in the preview, once it is found.
#[tauri::command]
pub fn recording_preview_face_path(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
) -> Result<Option<FacePathView>, String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  Ok(manager.sources.as_ref().and_then(|sources| {
    let path = sources.scenes.face_path.read().ok()?;
    path.as_ref().map(|path| FacePathView {
      step_ms: STEP_MS,
      points: path.points().iter().map(|&(x, y)| [x, y]).collect(),
    })
  }))
}

impl PlayerSources {
  /// Whether the face is wanted, as the camera or a scene that `follows` it
  /// wants it, and not yet found.
  pub(super) fn awaits_face(&self, follows: bool) -> bool {
    follows
      && self
        .scenes
        .face_path
        .read()
        .is_ok_and(|path| path.is_none())
  }
}

/// Starts finding the face in the camera of `artifact_id` where `wanted`.
/// It reads the recording open in the editor, so it starts on its own task
/// rather than under the preview's lock.
pub(super) fn find_face_when(wanted: bool, app: tauri::AppHandle, artifact_id: Option<u64>) {
  let Some(artifact_id) = artifact_id.filter(|_| wanted) else {
    return;
  };
  tauri::async_runtime::spawn_blocking(move || {
    if let Err(error) =
      crate::editor::camera_effects::face_tracking::find_recording_faces(app, artifact_id)
    {
      eprintln!("Could not start finding the face: {error}");
    }
  });
}

/// Hands the preview of `artifact_id` where the face is over its camera,
/// once it has been found, redrawing a paused frame that follows it.
pub(crate) fn set_face_path(app: &tauri::AppHandle, artifact_id: u64, path: Option<Arc<FacePath>>) {
  let Some(state) = tauri::Manager::try_state::<RecordingPreviewPlayerState>(app) else {
    return;
  };
  let Ok(mut manager) = state.0.lock() else {
    return;
  };
  let Some(sources) = manager
    .sources
    .as_ref()
    .filter(|_| manager.artifact_id == Some(artifact_id))
  else {
    return;
  };
  *sources
    .scenes
    .face_path
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = path;
  if let Err(error) = manager.redraw_paused_frame() {
    eprintln!("Could not redraw the preview with the face: {error}");
  }
  drop(manager);
  if let Err(error) = tauri::Emitter::emit(app, FACE_PATH_EVENT, ()) {
    eprintln!("Could not tell the editor the face was found: {error}");
  }
}
