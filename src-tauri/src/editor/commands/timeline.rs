// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Saving the recording's timeline edit, and what follows from it.

use tauri::{AppHandle, Manager};

use crate::editor::{camera_effects, project_look, timeline_edit, EditorArtifact, EditorState};

/// Saves the recording's timeline edit into its project, off the main
/// thread: writing the project takes milliseconds, and the main thread draws
/// every window.
#[tauri::command]
pub async fn set_recording_timeline_edit(
  app: AppHandle,
  artifact_id: u64,
  revision: u64,
  edit: screenwide_timeline_edit::RecordingTimelineEdit,
) -> Result<(), String> {
  let (project, duration_ms) = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Recording {
      id,
      project,
      duration_ms,
      ..
    }) = artifact.as_ref()
    else {
      return Err("There is no recording to edit".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    (project.clone(), *duration_ms)
  };
  tauri::async_runtime::spawn_blocking(move || {
    // The pictures the edit's images show go into the project with it, so
    // it shows them on another computer.
    project_look::carry_pictures(&project, &edit.annotation_clips);
    let before = timeline_edit::for_project(&project, artifact_id, duration_ms);
    let ranges = edit.camera_background_ranges.clone();
    let segments = edit.segments.clone();
    timeline_edit::persist(&project, artifact_id, revision, edit)?;
    // The camera's background follows its ranges, which the edit holds.
    camera_effects::edit_saved(
      &app,
      artifact_id,
      before.as_ref().map(|(_, edit)| edit),
      (&ranges, &segments),
    );
    Ok(())
  })
  .await
  .map_err(|error| error.to_string())?
}
