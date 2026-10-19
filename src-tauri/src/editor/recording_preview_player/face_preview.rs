// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The face the preview follows: found when the camera or a scene first asks
//! to follow it, and handed to the preview once it is there.

use super::*;
use screenwide_scenes::FacePath;

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
}
