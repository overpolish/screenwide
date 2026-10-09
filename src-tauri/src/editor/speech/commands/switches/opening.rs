// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording readied as it opens. Reduce noise, Vocal cleanup and Auto
//! volume all start on, so the first time a recording opens, or after a
//! change of make, their work is done in the background in the order each
//! needs the last: the noise taken out, the voice cleaned up from what that
//! leaves, then the voice as heard measured. Each fills its own bar.

use std::sync::Mutex;

use tauri::AppHandle;

use super::super::super::auto_volume::{self, AutoVolume};
use super::super::super::noise::{self, NoiseReduction};
use super::super::super::voice::{self, VocalCleanup};
use super::super::super::{duck, vad_model, SpeechSource};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, level_heard};

/// One recording readied at a time; asked again while it runs, the work
/// under way is left to finish.
static READYING: Mutex<()> = Mutex::new(());

/// Starts readying the recording `source` reads, if any switch that is on
/// still has work to do.
pub(super) fn ready(app: &AppHandle, artifact_id: u64, source: SpeechSource) {
  if !has_work(&source) {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let Ok(_readying) = READYING.try_lock() else {
      return;
    };
    let model = vad_model(&app).ok();
    let (folder, stream) = (&source.project_folder, source.microphone);
    if noise::choice(folder, stream) == NoiseReduction::On {
      if let Some(model) = model.clone() {
        let mut noise_bar = bar(&app, artifact_id, MicrophoneTool::Noise);
        let made = noise::prepare(&source.movie, stream, folder, model, &mut |fraction| {
          noise_bar.tell(fraction);
        });
        if let Err(error) = made {
          eprintln!("Could not take the microphone's noise out: {error}");
        }
      }
    }
    if voice::choice(folder, stream) == VocalCleanup::On {
      let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
      let made = voice::prepare(
        &source.movie,
        stream,
        folder,
        heard_denoised(&source),
        &mut |fraction| voice_bar.tell(fraction),
      );
      if let Err(error) = made {
        eprintln!("Could not clean up the microphone's voice: {error}");
      }
    }
    let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
    level_heard(&source, model, &mut |fraction| level_bar.tell(fraction));
    drop(level_bar);
    heard_changed(&app, artifact_id, &source);
  });
}

/// Whether the voice is heard from the file Reduce noise made.
fn heard_denoised(source: &SpeechSource) -> bool {
  let (folder, stream) = (&source.project_folder, source.microphone);
  noise::choice(folder, stream) == NoiseReduction::On && noise::is_made(folder, stream)
}

fn has_work(source: &SpeechSource) -> bool {
  let (folder, stream) = (&source.project_folder, source.microphone);
  let noise_on = noise::choice(folder, stream) == NoiseReduction::On;
  if noise_on && !noise::is_made(folder, stream) {
    return true;
  }
  if voice::choice(folder, stream) == VocalCleanup::On
    && !voice::is_made(folder, stream, heard_denoised(source))
  {
    return true;
  }
  let unducked = source
    .system
    .is_some_and(|system| !duck::is_made_for(folder, system, stream));
  auto_volume::choice(folder, stream) == AutoVolume::On
    && (auto_volume::is_unmeasured(folder, stream) || unducked)
}
