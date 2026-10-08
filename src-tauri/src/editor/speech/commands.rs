// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::noise::{self, NoiseReduction};
use super::{analysis, silences, vad_model, SpeechSource};
use crate::editor::recording_preview_player::{refresh_noise, PreviewNoise};

/// The event that tells the editor how far a microphone tool has got.
const PROGRESS_EVENT: &str = "editor://microphone-progress";

/// Which microphone tool is working.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum MicrophoneTool {
  Noise,
  Silences,
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

/// Whether the microphone of the recording open in the editor has its noise
/// taken out, as its project says.
#[tauri::command]
pub fn get_recording_noise_reduction(
  app: AppHandle,
  artifact_id: u64,
) -> Result<NoiseReduction, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  Ok(noise::choice(&source.project_folder, source.microphone))
}

/// Takes the microphone's noise out, or puts it back. Turning it on the first
/// time cleans the whole track, which takes a while for a long recording;
/// after that the switch is instant. Answers with what the project now says.
#[tauri::command]
pub async fn set_recording_noise_reduction(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<NoiseReduction, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  // Cleaning listens for speech too, but turning it off needs no model.
  let model = enabled.then(|| vad_model(&app)).transpose()?;
  let folder = source.project_folder.clone();
  let stream = source.microphone;
  let mut progress = reporter(&app, artifact_id, MicrophoneTool::Noise);
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let choice = if let Some(model) = model {
      noise::prepare(
        &source.movie,
        source.microphone,
        &source.project_folder,
        model,
        &mut progress,
      )?;
      NoiseReduction::On
    } else {
      NoiseReduction::Off
    };
    noise::keep(&source.project_folder, source.microphone, choice)?;
    Ok::<_, String>(choice)
  })
  .await
  .map_err(|error| error.to_string())??;
  refresh_noise(&app, PreviewNoise::for_project(&folder, &[stream]));
  // Read after the answer, since reading the track as recorded again can take
  // a few seconds on a long recording and the switch should not wait on it.
  let refreshing = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    if let Err(error) =
      crate::editor::recording_preview::refresh_waveform(&refreshing, artifact_id, stream)
    {
      eprintln!("Could not read the microphone's waveform again: {error}");
    }
  });
  Ok(choice)
}
