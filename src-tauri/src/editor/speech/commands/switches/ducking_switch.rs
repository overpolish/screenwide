// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Smart ducking switch. Turning it on makes the system audio's file,
//! its singing separated, and separates the singing in the songs, which
//! needs the model downloaded; turning it off lets both play as they are.

use tauri::{AppHandle, Emitter};

use super::super::super::smart_ducking::{self, SmartDucking};
use super::super::super::{vad_model, SpeechSource};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, opening, ready, singing};

/// Whether the system audio and the songs of the recording open in the
/// editor make way for the voice, as its project says. A recording starts
/// with it on, so the first time one opens its switches' work is started in
/// the background.
#[tauri::command]
pub fn get_recording_smart_ducking(
  app: AppHandle,
  artifact_id: u64,
) -> Result<SmartDucking, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let choice = smart_ducking::choice(&source.project_folder, source.microphone);
  opening::ready(&app, artifact_id, source);
  Ok(choice)
}

/// Makes the system audio and the songs make way for the voice, their
/// singing out of its way, or lets them play as they are. Answers with what
/// the project now says.
#[tauri::command]
pub async fn set_recording_smart_ducking(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<SmartDucking, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  if enabled && smart_ducking::model(&app).is_none() {
    return Err("Download the Smart ducking model first".to_owned());
  }
  let choice = if enabled {
    SmartDucking::On
  } else {
    SmartDucking::Off
  };
  smart_ducking::keep(&source.project_folder, source.microphone, choice)?;
  // The songs are laid out as the preview starts playing: a preview playing
  // starts again where it is, the songs making way or not.
  let _ = app.emit("editor://smart-ducking", choice);
  if enabled {
    crate::editor::sound_library::separate_open_songs(&app);
  }
  let model = vad_model(&app).ok();
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  // Read after the choice is kept, so it makes the file the switch now asks for.
  let mut singing = singing(&app, artifact_id, &source);
  let source = tauri::async_runtime::spawn_blocking(move || {
    if let Err(error) = ready(&source, model, &mut singing, &mut level_bar) {
      if enabled {
        let _ = smart_ducking::keep(&source.project_folder, source.microphone, SmartDucking::Off);
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
