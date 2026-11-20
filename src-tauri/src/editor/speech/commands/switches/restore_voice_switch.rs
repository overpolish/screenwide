// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Restore voice switch. Turning it on restores the microphone the first
//! time, which needs the model downloaded; turning it off brings back
//! Reduce noise and Vocal cleanup, readied where they are on.

use tauri::AppHandle;

use super::super::super::{ready_voice, vad_model, SpeechSource, VoiceSteps};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, level_heard, opening, singing};
use screenwide_speech::restore_voice::{self, RestoreVoice};

/// Whether the microphone of the recording open in the editor is restored,
/// as its project says. A recording starts with it on, so the first time one
/// opens its switches' work is started in the background.
#[tauri::command]
pub fn get_recording_restore_voice(
  app: AppHandle,
  artifact_id: u64,
) -> Result<RestoreVoice, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let choice = restore_voice::choice(&source.project_folder, source.microphone);
  opening::ready(&app, artifact_id, source);
  Ok(choice)
}

/// Restores the microphone, or brings back how Reduce noise and Vocal
/// cleanup have it. Answers with what the project now says.
#[tauri::command]
pub async fn set_recording_restore_voice(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<RestoreVoice, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  if enabled && crate::editor::speech::restore_voice_model(&app).is_none() {
    return Err("Download the Restore voice model first".to_owned());
  }
  let vad = vad_model(&app).ok();
  let mut restore_bar = bar(&app, artifact_id, MicrophoneTool::RestoreVoice);
  let mut noise_bar = bar(&app, artifact_id, MicrophoneTool::Noise);
  let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  let mut singing = singing(&app, artifact_id, &source);
  let worker = app.clone();
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let (folder, stream) = (&source.project_folder, source.microphone);
    let choice = if enabled {
      RestoreVoice::On
    } else {
      RestoreVoice::Off
    };
    restore_voice::keep(folder, stream, choice)?;
    ready_voice(
      &worker,
      folder,
      (&source.movie, stream),
      vad.clone(),
      VoiceSteps {
        restore: &mut restore_bar,
        noise: &mut noise_bar,
        voice: &mut voice_bar,
      },
    )
    .inspect_err(|_| {
      if enabled {
        let _ = restore_voice::keep(folder, stream, RestoreVoice::Off);
      }
    })?;
    level_heard(&source, vad, &mut singing, &mut level_bar);
    Ok::<_, String>((choice, source))
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &choice.1);
  Ok(choice.0)
}
