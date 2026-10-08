// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How likely each moment of the microphone track is to be speech, as the
//! voice activity model hears it. A recording never changes once made, so
//! the answer is kept in the project and read back after the first listen.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::SAMPLE_RATE;
use serde::{Deserialize, Serialize};

use crate::transcription::runner::{decode_audio_stream, Transcriber};

/// Bumped when what is kept stops matching what a listen gives, so an old
/// file is listened again rather than read wrongly.
const FORMAT_VERSION: u16 = 1;

/// The chance of speech in each window of the track, in order.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpeechMap {
  pub probabilities: Vec<f32>,
  pub window_ms: f64,
}

impl SpeechMap {
  /// Where window `index` starts, in milliseconds of the recording.
  pub(super) fn start_ms(&self, index: usize) -> f64 {
    index as f64 * self.window_ms
  }
}

/// What is kept: each chance in 256 steps, which is finer than any choice
/// made from it and a quarter of the size of the floats.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Kept {
  version: u16,
  stream: usize,
  window_samples: u32,
  levels: Vec<u8>,
}

fn kept_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("speech-{stream}.json"))
}

/// The speech in the `stream`th audio track of `movie`, read from the
/// project where it was kept, or listened for with the model at `model` and
/// kept there, telling `progress` how far listening has got.
pub(super) fn speech_map(
  movie: &Path,
  stream: usize,
  project_folder: &Path,
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<SpeechMap, String> {
  // One listen at a time: a second tool asking while the first listens waits
  // and then reads what the first kept.
  static LISTENING: Mutex<()> = Mutex::new(());
  let _listening = LISTENING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let path = kept_path(project_folder, stream);
  if let Some(map) = read_kept(&path, stream) {
    return Ok(map);
  }
  // Said before the track is read, so the editor shows its bar only when
  // there is listening to wait for, and shows it from the start.
  progress(0.0);
  let samples = decode_audio_stream(movie, stream, SAMPLE_RATE)?;
  let speech = Transcriber::start()?.detect_speech(model, &samples, progress)?;
  let kept = Kept {
    levels: speech
      .probabilities
      .iter()
      .map(|chance| (chance.clamp(0.0, 1.0) * 255.0).round() as u8)
      .collect(),
    stream,
    version: FORMAT_VERSION,
    window_samples: speech.window_samples,
  };
  // A listen that cannot be kept still answers this time.
  if let Err(error) = std::fs::write(
    &path,
    serde_json::to_vec(&kept).map_err(|error| error.to_string())?,
  ) {
    eprintln!("Could not keep what the microphone says: {error}");
  }
  Ok(SpeechMap {
    probabilities: speech.probabilities,
    window_ms: window_ms(speech.window_samples),
  })
}

fn read_kept(path: &Path, stream: usize) -> Option<SpeechMap> {
  let kept: Kept = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
  (kept.version == FORMAT_VERSION && kept.stream == stream && kept.window_samples > 0).then(|| {
    SpeechMap {
      probabilities: kept
        .levels
        .iter()
        .map(|&level| f32::from(level) / 255.0)
        .collect(),
      window_ms: window_ms(kept.window_samples),
    }
  })
}

fn window_ms(window_samples: u32) -> f64 {
  f64::from(window_samples) * 1_000.0 / f64::from(SAMPLE_RATE)
}
