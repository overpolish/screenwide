// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One source of a call's notes: its audio, laid out from the notes' start,
//! and where it pauses. A source sends nothing while it is silent on some
//! systems, so each run of samples is placed at its own time, the gap before
//! it filled with silence, and both tracks keep to one clock.

use std::path::Path;
use std::sync::Mutex;

use screenwide_transcriber::SAMPLE_RATE;
use transcribe_cpp::{Model, VadSession};

use super::super::found;
use super::super::stretches::{Pauses, Said, Stretch};
use super::super::writer::lock;

/// A track's audio, as 16-bit samples from the notes' start: half the memory
/// of the models' own samples, for a call that may run for hours.
pub type Audio = Mutex<Vec<i16>>;

pub struct Track {
  vad: VadSession,
  pauses: Pauses,
}

impl Track {
  /// A track finding its pauses with the voice activity model at `vad`.
  pub fn new(vad: &Path) -> Result<Self, String> {
    let vad = Model::load(vad)
      .and_then(|model| model.vad_session())
      .map_err(|error| format!("Could not load the voice activity model: {error}"))?;
    Ok(Self {
      vad,
      pauses: Pauses::default(),
    })
  }

  /// Hears `samples`, starting `at_ms` into the notes, into `audio`,
  /// answering the stretches a pause has just ended. Samples from before what
  /// was already heard follow on from it.
  pub fn feed(
    &mut self,
    audio: &Audio,
    at_ms: u64,
    samples: &[i16],
  ) -> Result<Vec<Stretch>, String> {
    let at = usize::try_from(at_ms * u64::from(SAMPLE_RATE) / 1_000).unwrap_or(usize::MAX);
    let pcm: Vec<f32> = {
      let mut audio = lock(audio);
      let gap = at.saturating_sub(audio.len());
      let new_len = audio.len() + gap;
      audio.resize(new_len, 0);
      audio.extend_from_slice(samples);
      std::iter::repeat_n(0.0, gap)
        .chain(samples.iter().map(|&sample| f32::from(sample) / 32_768.0))
        .collect()
    };
    let heard = self
      .vad
      .stream_feed(&pcm)
      .map_err(|error| format!("Could not hear the notes: {error}"))?;
    Ok(stretches(found(&mut self.pauses, &heard)))
  }

  /// The stretches still open as the notes end.
  pub fn finish(&mut self) -> Vec<Stretch> {
    let mut last = self
      .vad
      .stream_flush()
      .map(|heard| stretches(found(&mut self.pauses, &heard)))
      .unwrap_or_default();
    last.extend(self.pauses.end());
    last
  }
}

/// The stretches of `said`: notes are written stretch by stretch and timed
/// by what each was written as, so they take no runs.
fn stretches(said: Vec<Said>) -> Vec<Stretch> {
  said.into_iter().map(|said| said.stretch).collect()
}
