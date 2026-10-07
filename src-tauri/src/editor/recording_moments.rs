// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::{EditorArtifact, EditorState};

/// A moment as the timeline shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMoment {
  pub color: String,
  pub kind_id: String,
  pub name: String,
  /// On the recording's own timeline, before any edit.
  pub source_ms: f64,
}

/// The moments placed in the recording open in the editor, in the order
/// they were placed. A recording without any, or whose moments file is gone
/// or unreadable, has none.
#[tauri::command]
pub async fn get_recording_moments(
  app: AppHandle,
  artifact_id: u64,
) -> Result<Vec<RecordingMoment>, String> {
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
  tauri::async_runtime::spawn_blocking(move || Ok(project.map(read).unwrap_or_default()))
    .await
    .map_err(|error| error.to_string())?
}

fn read(project: PathBuf) -> Vec<RecordingMoment> {
  let path = crate::project::read(&project).ok().and_then(|manifest| {
    let name = manifest.recorded().ok()?.media.moments.clone()?;
    crate::project::resolve(&project, &name).ok()
  });
  let Some(path) = path else {
    return Vec::new();
  };
  crate::moments::read(&path)
    .inspect_err(|error| eprintln!("Could not read this recording's moments: {error}"))
    .unwrap_or_default()
    .into_iter()
    .map(|moment| RecordingMoment {
      color: moment.color,
      kind_id: moment.kind_id,
      name: moment.name,
      source_ms: moment.timestamp_us as f64 / 1_000.0,
    })
    .collect()
}
