// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone's switches: Reduce noise, Vocal cleanup and Auto volume.
//! Each does its work the first time it is turned a way, which can take a
//! while on a long recording, and is instant after. Turning one can call for
//! another's work too: Vocal cleanup is made again from what Reduce noise
//! leaves, and Auto volume measures whatever is heard. Each kind of work
//! fills its own switch's bar, whichever switch called for it.

use tauri::AppHandle;

use super::super::auto_volume::{self, AutoVolume};
use super::super::noise::{self, NoiseReduction};
use super::super::voice::{self, VocalCleanup};
use super::super::{vad_model, SpeechSource};
use super::{reporter, MicrophoneTool};
use crate::editor::recording_preview_player::{refresh_processing, PreviewProcessing};

/// A tool's progress bar. Once anything has been told, the bar is told it is
/// full when this goes, whether the work finished or failed, so it never
/// hangs part-way.
struct Bar<F: FnMut(f32)> {
  report: F,
  told: bool,
}

impl<F: FnMut(f32)> Bar<F> {
  fn tell(&mut self, fraction: f32) {
    self.told = true;
    (self.report)(fraction);
  }
}

impl<F: FnMut(f32)> Drop for Bar<F> {
  fn drop(&mut self) {
    if self.told {
      (self.report)(1.0);
    }
  }
}

fn bar(app: &AppHandle, artifact_id: u64, tool: MicrophoneTool) -> Bar<impl FnMut(f32)> {
  Bar {
    report: reporter(app, artifact_id, tool),
    told: false,
  }
}

/// Measures the track as it is heard now for Auto volume, where it is on. A
/// measure that fails leaves the track heard unleveled rather than undoing
/// the switch that called for it.
fn level_heard(source: &SpeechSource, progress: &mut dyn FnMut(f32)) {
  if auto_volume::choice(&source.project_folder, source.microphone) != AutoVolume::On {
    return;
  }
  if let Err(error) = auto_volume::measure(
    &source.project_folder,
    source.microphone,
    &source.movie,
    source.duration_ms,
    progress,
  ) {
    eprintln!("Could not measure the microphone's volume: {error}");
  }
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

/// Takes the microphone's noise out, or puts it back. Answers with what the
/// project now says.
#[tauri::command]
pub async fn set_recording_noise_reduction(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<NoiseReduction, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  // Cleaning listens for speech too, but turning it off needs no model.
  let model = enabled.then(|| vad_model(&app)).transpose()?;
  let mut noise_bar = bar(&app, artifact_id, MicrophoneTool::Noise);
  let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let (folder, stream) = (&source.project_folder, source.microphone);
    if let Some(model) = model {
      noise::prepare(&source.movie, stream, folder, model, &mut |fraction| {
        noise_bar.tell(fraction);
      })?;
    }
    if voice::choice(folder, stream) == VocalCleanup::On {
      voice::prepare(&source.movie, stream, folder, enabled, &mut |fraction| {
        voice_bar.tell(fraction);
      })?;
    }
    let choice = if enabled {
      NoiseReduction::On
    } else {
      NoiseReduction::Off
    };
    noise::keep(folder, stream, choice)?;
    level_heard(&source, &mut |fraction| level_bar.tell(fraction));
    Ok::<_, String>((choice, source))
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &choice.1);
  Ok(choice.0)
}

/// Whether the microphone of the recording open in the editor has its voice
/// cleaned up, as its project says.
#[tauri::command]
pub fn get_recording_vocal_cleanup(
  app: AppHandle,
  artifact_id: u64,
) -> Result<VocalCleanup, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  Ok(voice::choice(&source.project_folder, source.microphone))
}

/// Cleans up the microphone's voice as it is heard now, with or without its
/// noise, or puts it back as it was. Answers with what the project now says.
#[tauri::command]
pub async fn set_recording_vocal_cleanup(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<VocalCleanup, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let (folder, stream) = (&source.project_folder, source.microphone);
    let choice = if enabled {
      let denoised =
        noise::choice(folder, stream) == NoiseReduction::On && noise::is_made(folder, stream);
      voice::prepare(&source.movie, stream, folder, denoised, &mut |fraction| {
        voice_bar.tell(fraction);
      })?;
      VocalCleanup::On
    } else {
      VocalCleanup::Off
    };
    voice::keep(folder, stream, choice)?;
    level_heard(&source, &mut |fraction| level_bar.tell(fraction));
    Ok::<_, String>((choice, source))
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &choice.1);
  Ok(choice.0)
}

/// Whether the microphone of the recording open in the editor is brought to
/// a steady loudness, as its project says. A recording starts with it on, so
/// the first time one opens, or whenever what is heard has not been
/// measured, the measure is taken in the background, with its progress told.
#[tauri::command]
pub fn get_recording_auto_volume(app: AppHandle, artifact_id: u64) -> Result<AutoVolume, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let (folder, stream) = (&source.project_folder, source.microphone);
  let choice = auto_volume::choice(folder, stream);
  if choice == AutoVolume::On && auto_volume::is_unmeasured(folder, stream) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
      let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
      level_heard(&source, &mut |fraction| level_bar.tell(fraction));
      drop(level_bar);
      heard_changed(&app, artifact_id, &source);
    });
  }
  Ok(choice)
}

/// Brings the microphone to a steady loudness, or plays it as it is. Turning
/// it on the first time for what is heard measures it. Answers with what the
/// project now says.
#[tauri::command]
pub async fn set_recording_auto_volume(
  app: AppHandle,
  artifact_id: u64,
  enabled: bool,
) -> Result<AutoVolume, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
  let choice = tauri::async_runtime::spawn_blocking(move || {
    let (folder, stream) = (&source.project_folder, source.microphone);
    let choice = if enabled {
      auto_volume::measure(
        folder,
        stream,
        &source.movie,
        source.duration_ms,
        &mut |fraction| level_bar.tell(fraction),
      )?;
      AutoVolume::On
    } else {
      AutoVolume::Off
    };
    auto_volume::keep(folder, stream, choice)?;
    Ok::<_, String>((choice, source))
  })
  .await
  .map_err(|error| error.to_string())??;
  heard_changed(&app, artifact_id, &choice.1);
  Ok(choice.0)
}

/// Tells the preview how the microphone is heard now, and reads its
/// waveform again.
fn heard_changed(app: &AppHandle, artifact_id: u64, source: &SpeechSource) {
  let stream = source.microphone;
  refresh_processing(
    app,
    PreviewProcessing::for_project(&source.project_folder, &[stream]),
  );
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
}
