// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Studio ducking switch. Turning it on makes the system audio's file
//! again with its singing separated, which needs the model downloaded;
//! turning it off makes it again making way as a whole. Either is work only
//! where Auto volume is on, since only then does the system audio make way.

use tauri::AppHandle;

use super::super::super::studio_ducking::{self, StudioDucking};
use super::super::super::{vad_model, SpeechSource};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, opening, ready, singing};

/// Whether the singing in the system audio of the recording open in the
/// editor gets out of the voice's way, as its project says. A recording
/// starts with it on, so the first time one opens its switches' work is
/// started in the background.
#[tauri::command]
pub fn get_recording_studio_ducking(
  app: AppHandle,
  artifact_id: u64,
) -> Result<StudioDucking, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let choice = studio_ducking::choice(&source.project_folder, source.microphone);
  opening::ready(&app, artifact_id, source);
  Ok(choice)
}

/// Moves the singing in the system audio out of the voice's way, or lets the
/// system audio make way as a whole. Answers with what the project now says.
#[tauri::command]
pub async fn set_recording_studio_ducking(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<StudioDucking, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  if enabled && studio_ducking::model(&app).is_none() {
    return Err("Download the Studio ducking model first".to_owned());
  }
  let choice = if enabled {
    StudioDucking::On
  } else {
    StudioDucking::Off
  };
  studio_ducking::keep(&source.project_folder, source.microphone, choice)?;
  let model = vad_model(&app).ok();
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  // Read after the choice is kept, so it makes the file the switch now asks for.
  let mut singing = singing(&app, artifact_id, &source);
  let source = tauri::async_runtime::spawn_blocking(move || {
    let level = &mut |fraction| level_bar.tell(fraction);
    if let Err(error) = ready(&source, model, &mut singing, level) {
      if enabled {
        let _ = studio_ducking::keep(
          &source.project_folder,
          source.microphone,
          StudioDucking::Off,
        );
      }
      return Err(error);
    }
    Ok::<_, String>(source)
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &source);
  Ok(choice)
}
