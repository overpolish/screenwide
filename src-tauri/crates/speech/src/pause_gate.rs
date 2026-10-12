// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pauses of a cleaned track turned down. The enhancer keeps whatever it
//! takes for voice, so a breath or a hiss between words can pass, and the
//! loudness gain that follows lifts it as well. Where the speech map hears
//! no speech, the track is held well below the voice, with the same margins
//! around speech that Remove silences keeps, and eased in and out so no
//! word's first or last sound is clipped.

use super::analysis::SpeechMap;
use super::silences::{speech_ranges, KEEP_AFTER_SPEECH_MS, KEEP_BEFORE_SPEECH_MS};

/// How far below the voice a pause is held: 40 dB, low enough that what the
/// enhancer let through is gone, without the dead stop of true silence.
const PAUSE_GAIN: f32 = 0.01;
/// How long the gain takes to go between voice and pause.
const FADE_MS: f64 = 80.0;

pub(super) struct PauseGate {
  /// The stretches at full gain, in samples, sorted and apart.
  open: Vec<(u64, u64)>,
  fade: u64,
  /// The first stretch that ends after the last sample asked about.
  next: usize,
}

impl PauseGate {
  /// The gate for a track at `rate` whose speech `speech` maps, `length`
  /// samples long.
  pub(super) fn new(speech: &SpeechMap, rate: u32, length: u64) -> Self {
    let per_ms = f64::from(rate) / 1_000.0;
    let duration_ms = length as f64 / per_ms;
    let mut open: Vec<(u64, u64)> = Vec::new();
    for (start, end) in speech_ranges(speech, duration_ms) {
      let start = ((start - KEEP_BEFORE_SPEECH_MS).max(0.0) * per_ms) as u64;
      let end = ((end + KEEP_AFTER_SPEECH_MS) * per_ms) as u64;
      match open.last_mut() {
        Some(last) if start <= last.1 => last.1 = last.1.max(end),
        _ => open.push((start, end)),
      }
    }
    Self {
      fade: ((FADE_MS * per_ms) as u64).max(1),
      next: 0,
      open,
    }
  }

  /// Turns the pauses in `samples` down, where they start `offset` samples
  /// into the track. Each call must start where the last ended.
  pub(super) fn apply(&mut self, offset: u64, samples: &mut [f32]) {
    for (index, sample) in samples.iter_mut().enumerate() {
      *sample *= self.gain(offset + index as u64);
    }
  }

  fn gain(&mut self, at: u64) -> f32 {
    while self.open.get(self.next).is_some_and(|&(_, end)| end <= at) {
      self.next += 1;
    }
    let until_next = self
      .open
      .get(self.next)
      .map_or(u64::MAX, |&(start, _)| start.saturating_sub(at));
    let since_last = self
      .next
      .checked_sub(1)
      .map_or(u64::MAX, |last| at + 1 - self.open[last].1);
    let away = until_next.min(since_last);
    if away >= self.fade {
      return PAUSE_GAIN;
    }
    let open = 0.5 * (1.0 + (std::f32::consts::PI * away as f32 / self.fade as f32).cos());
    PAUSE_GAIN + (1.0 - PAUSE_GAIN) * open
  }
}
