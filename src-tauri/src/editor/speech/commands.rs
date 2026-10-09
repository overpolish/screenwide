// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod switches;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::{analysis, silences, vad_model, SpeechSource};

/// The event that tells the editor how far a microphone tool has got.
const PROGRESS_EVENT: &str = "editor://microphone-progress";

/// Which microphone tool is working.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MicrophoneTool {
  AutoVolume,
  Noise,
  Silences,
  StudioSound,
  Voice,
}

/// How far a microphone tool has got with the recording, 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MicrophoneProgress {
  pub artifact_id: u64,
  pub fraction: f32,
  pub tool: MicrophoneTool,
}

/// A progress callback that tells the editor about `tool` on `artifact_id`.
fn reporter(app: &AppHandle, artifact_id: u64, tool: MicrophoneTool) -> impl FnMut(f32) {
  let app = app.clone();
  move |fraction| {
    let _ = app.emit(
      PROGRESS_EVENT,
      MicrophoneProgress {
        artifact_id,
        fraction,
        tool,
      },
    );
  }
}

/// A stretch of the recording Remove silences would cut, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SilenceCut {
  pub start_ms: u64,
  pub end_ms: u64,
}

/// The long pauses in the speech of the recording open in the editor.
/// Listening runs the first time and is kept for the next.
#[tauri::command]
pub async fn plan_recording_silences(
  app: AppHandle,
  artifact_id: u64,
) -> Result<Vec<SilenceCut>, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let model = vad_model(&app)?;
  let mut progress = reporter(&app, artifact_id, MicrophoneTool::Silences);
  tauri::async_runtime::spawn_blocking(move || {
    let speech = analysis::speech_map(
      &source.movie,
      source.microphone,
      &source.project_folder,
      model,
      &mut progress,
    )?;
    Ok(
      silences::plan(&speech, source.duration_ms)
        .into_iter()
        .map(|(start_ms, end_ms)| SilenceCut { end_ms, start_ms })
        .collect(),
    )
  })
  .await
  .map_err(|error| error.to_string())?
}
