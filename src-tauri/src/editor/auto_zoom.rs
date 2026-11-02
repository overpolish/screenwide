// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The auto zooms `screenwide_auto_zoom` plans, for a freshly captured
//! recording and for the Scene panel's request to make them again.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::{EditorArtifact, EditorState};
use screenwide_auto_zoom::{plan, VisibleArea};
use screenwide_scenes::RecordingSceneClip;

/// The auto zooms a freshly captured recording starts with: none where they
/// are turned off in Settings or there is no cursor to follow.
pub(in crate::editor) fn for_new_recording(
  app: &AppHandle,
  cursor_path: Option<&Path>,
  keyboard_path: Option<&Path>,
  duration_ms: u64,
) -> Vec<RecordingSceneClip> {
  let Some(cursor_path) = cursor_path.filter(|_| crate::settings::current(app).auto_zoom) else {
    return Vec::new();
  };
  plan(cursor_path, keyboard_path, duration_ms, VisibleArea::WHOLE).unwrap_or_else(|error| {
    eprintln!("Could not make this recording's auto zooms: {error}");
    Vec::new()
  })
}

/// The auto zooms for the recording open in the editor, for the Scene panel
/// to put in place of the ones it has. They are timed on the whole
/// recording; the panel carries them onto the timeline it edits.
#[tauri::command]
pub async fn plan_recording_auto_zoom(
  app: AppHandle,
  artifact_id: u64,
  visible: VisibleArea,
) -> Result<Vec<RecordingSceneClip>, String> {
  let (cursor, keyboard, duration_ms): (PathBuf, Option<PathBuf>, u64) = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Recording {
      id,
      cursor,
      keyboard,
      duration_ms,
      ..
    }) = artifact.as_ref()
    else {
      return Err("There is no recording to zoom into".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    let cursor = cursor
      .as_ref()
      .ok_or_else(|| "This recording has no cursor movement".to_owned())?;
    (
      cursor.path.clone(),
      keyboard.as_ref().map(|keyboard| keyboard.path.clone()),
      *duration_ms,
    )
  };
  tauri::async_runtime::spawn_blocking(move || {
    plan(&cursor, keyboard.as_deref(), duration_ms, visible)
  })
  .await
  .map_err(|error| error.to_string())?
}
