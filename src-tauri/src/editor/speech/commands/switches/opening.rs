// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording readied as it opens. The microphone's switches all start
//! on, so the first time a recording opens, or after a change of make, their
//! work is done in the background in the order each needs the last: Studio
//! sound's rebuilt voice where its model is downloaded, or else the noise
//! taken out and the voice cleaned up from what that leaves, then the voice
//! as heard measured. Each fills its own bar.

use std::sync::Mutex;

use tauri::AppHandle;

use super::super::super::auto_volume::{self, AutoVolume};
use super::super::super::noise::{self, NoiseReduction};
use super::super::super::studio::{self, StudioSound};
use super::super::super::voice::{self, VocalCleanup};
use super::super::super::{duck, ready_voice, vad_model, SpeechSource, VoiceProgress};
use super::super::MicrophoneTool;
use super::{bar, heard_changed, level_heard};

/// One recording readied at a time; asked again while it runs, the work
/// under way is left to finish.
static READYING: Mutex<()> = Mutex::new(());

/// Starts readying the recording `source` reads, if any switch that is on
/// still has work to do.
pub(super) fn ready(app: &AppHandle, artifact_id: u64, source: SpeechSource) {
  if !has_work(app, &source) {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let Ok(_readying) = READYING.try_lock() else {
      return;
    };
    let vad = vad_model(&app).ok();
    let mut studio_bar = bar(&app, artifact_id, MicrophoneTool::StudioSound);
    let mut noise_bar = bar(&app, artifact_id, MicrophoneTool::Noise);
    let mut voice_bar = bar(&app, artifact_id, MicrophoneTool::Voice);
    let made = ready_voice(
      &app,
      &source.project_folder,
      (&source.movie, source.microphone),
      vad.clone(),
      VoiceProgress {
        studio: &mut |fraction| studio_bar.tell(fraction),
        noise: &mut |fraction| noise_bar.tell(fraction),
        voice: &mut |fraction| voice_bar.tell(fraction),
      },
    );
    if let Err(error) = made {
      eprintln!("Could not ready the microphone's voice: {error}");
    }
    drop((studio_bar, noise_bar, voice_bar));
    let mut level_bar = bar(&app, artifact_id, MicrophoneTool::AutoVolume);
    level_heard(&source, vad, &mut |fraction| level_bar.tell(fraction));
    drop(level_bar);
    heard_changed(&app, artifact_id, &source);
  });
}

fn has_work(app: &AppHandle, source: &SpeechSource) -> bool {
  let (folder, stream) = (&source.project_folder, source.microphone);
  let studio_on = studio::choice(folder, stream) == StudioSound::On;
  if studio_on && studio::model(app).is_some() {
    if !studio::is_made(folder, stream) {
      return true;
    }
  } else {
    let noise_on = noise::choice(folder, stream) == NoiseReduction::On;
    if noise_on && !noise::is_made(folder, stream) {
      return true;
    }
    let denoised = noise_on && noise::is_made(folder, stream);
    if voice::choice(folder, stream) == VocalCleanup::On
      && !voice::is_made(folder, stream, denoised)
    {
      return true;
    }
  }
  let unducked = source
    .system
    .is_some_and(|system| !duck::is_made_for(folder, system, stream));
  auto_volume::choice(folder, stream) == AutoVolume::On
    && (auto_volume::is_unmeasured(folder, stream) || unducked)
}
