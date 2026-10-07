// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{ipc::Response, AppHandle, Manager};

use super::{EditorArtifact, EditorState};

/// How many levels a note's waveform is drawn from.
const NOTE_WAVEFORM_POINTS: usize = 160;

/// A moment as the timeline shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMoment {
  pub color: String,
  /// Which moment it is, counting from zero in the order they were placed.
  pub index: usize,
  pub kind_id: String,
  pub name: String,
  pub note: Option<RecordingMomentNote>,
  /// On the recording's own timeline, before any edit.
  pub source_ms: f64,
}

/// The voice note recorded with a moment.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMomentNote {
  pub duration_ms: u64,
  /// Peak levels from 0 to 1, for drawing.
  pub waveform: Vec<f32>,
}

/// The moments file of the recording open in the editor as `artifact_id`.
fn moments_path(app: &AppHandle, artifact_id: u64) -> Result<Option<PathBuf>, String> {
  let project = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(recording @ EditorArtifact::Recording { id, .. }) = artifact.as_ref() else {
      return Err("There is no recording open".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    recording.project().map(Path::to_path_buf)
  };
  Ok(project.and_then(|project| {
    let name = crate::project::read(&project)
      .ok()?
      .recorded()
      .ok()?
      .media
      .moments
      .clone()?;
    crate::project::resolve(&project, &name).ok()
  }))
}

/// The moments placed in the recording open in the editor, in the order
/// they were placed. A recording without any, or whose moments file is gone
/// or unreadable, has none.
#[tauri::command]
pub async fn get_recording_moments(
  app: AppHandle,
  artifact_id: u64,
) -> Result<Vec<RecordingMoment>, String> {
  let path = moments_path(&app, artifact_id)?;
  tauri::async_runtime::spawn_blocking(move || Ok(path.map(|path| read(&path)).unwrap_or_default()))
    .await
    .map_err(|error| error.to_string())?
}

fn read(path: &Path) -> Vec<RecordingMoment> {
  let folder = path.parent().unwrap_or(Path::new(""));
  crate::moments::read(path)
    .inspect_err(|error| eprintln!("Could not read this recording's moments: {error}"))
    .unwrap_or_default()
    .into_iter()
    .enumerate()
    .map(|(index, moment)| RecordingMoment {
      color: moment.color,
      index,
      kind_id: moment.kind_id,
      name: moment.name,
      note: moment.note_duration_ms.and_then(|duration_ms| {
        let waveform = crate::moments::note_audio::waveform(
          &crate::moments::note_path(folder, index),
          NOTE_WAVEFORM_POINTS,
        )
        .ok()?;
        Some(RecordingMomentNote {
          duration_ms,
          waveform,
        })
      }),
      source_ms: moment.timestamp_us as f64 / 1_000.0,
    })
    .collect()
}

/// The voice note of the `moment`th moment of the recording open in the
/// editor, as WAV bytes. Only a note the moments file lists is read.
#[tauri::command]
pub async fn get_recording_moment_note(
  app: AppHandle,
  artifact_id: u64,
  moment: usize,
) -> Result<Response, String> {
  let path =
    moments_path(&app, artifact_id)?.ok_or_else(|| "This recording has no moments".to_owned())?;
  tauri::async_runtime::spawn_blocking(move || {
    let listed = crate::moments::read(&path)?
      .get(moment)
      .is_some_and(|recorded| recorded.note_duration_ms.is_some());
    if !listed {
      return Err("That moment has no voice note".to_owned());
    }
    let folder = path.parent().unwrap_or(Path::new(""));
    std::fs::read(crate::moments::note_path(folder, moment))
      .map(Response::new)
      .map_err(|error| error.to_string())
  })
  .await
  .map_err(|error| error.to_string())?
}
