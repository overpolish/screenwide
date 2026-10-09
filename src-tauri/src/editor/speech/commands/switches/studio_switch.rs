// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Studio sound switch. Turning it on rebuilds the microphone the first
//! time, which needs the model downloaded; turning it off brings back
//! Reduce noise and Vocal cleanup, readied where they are on.

use tauri::AppHandle;

use super::super::super::studio::{self, StudioSound};
use super::super::super::{ready_voice, vad_model, SpeechSource, VoiceProgress};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, level_heard, opening};

/// Whether the microphone of the recording open in the editor is rebuilt as
/// a studio recording, as its project says. A recording starts with it on,
/// so the first time one opens its switches' work is started in the
/// background.
#[tauri::command]
pub fn get_recording_studio_sound(app: AppHandle, artifact_id: u64) -> Result<StudioSound, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let choice = studio::choice(&source.project_folder, source.microphone);
  opening::ready(&app, artifact_id, source);
  Ok(choice)
}

/// Rebuilds the microphone as a studio recording, or brings back how Reduce
/// noise and Vocal cleanup have it. Answers with what the project now says.
#[tauri::command]
pub async fn set_recording_studio_sound(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<StudioSound, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  if enabled && studio::model(&app).is_none() {
    return Err("Download the Studio sound model first".to_owned());
  }
  let vad = vad_model(&app).ok();
  let mut studio_bar = bar(&app, artifact_id, MicrophoneTool::StudioSound);
  let mut noise_bar = bar(&app, artifact_id, MicrophoneTool::Noise);
  let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  let worker = app.clone();
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let (folder, stream) = (&source.project_folder, source.microphone);
    let choice = if enabled {
      StudioSound::On
    } else {
      StudioSound::Off
    };
    studio::keep(folder, stream, choice)?;
    ready_voice(
      &worker,
      folder,
      (&source.movie, stream),
      vad.clone(),
      VoiceProgress {
        studio: &mut |fraction| studio_bar.tell(fraction),
        noise: &mut |fraction| noise_bar.tell(fraction),
        voice: &mut |fraction| voice_bar.tell(fraction),
      },
    )
    .inspect_err(|_| {
      if enabled {
        let _ = studio::keep(folder, stream, StudioSound::Off);
      }
    })?;
    level_heard(&source, vad, &mut |fraction| level_bar.tell(fraction));
    Ok::<_, String>((choice, source))
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &choice.1);
  Ok(choice.0)
}
