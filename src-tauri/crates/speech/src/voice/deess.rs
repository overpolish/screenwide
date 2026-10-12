// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Harsh "s" sounds softened. Where the voice above 5 kHz grows loud against
//! the voice below it, as it does on a sibilant, that part is turned down by
//! up to 6 dB, and only while someone is speaking, so the hiss of a room
//! between words is not pumped.
//!
//! It judges the balance within the voice rather than a fixed level, so a
//! quiet microphone is treated the same as a loud one. FFmpeg's `deesser`
//! was tried first: it left quiet tracks alone, and blew up on a track
//! whose samples went past full scale.

use super::biquad::Split;

const SPLIT_HZ: f64 = 5_000.0;
/// The balance a sibilant is softened toward: the voice above the split at
/// a quarter of the energy of the voice below it.
const BALANCE: f32 = 0.25;
/// How much of a sibilant's excess over [`BALANCE`] is taken off, in dB.
const SHARE: f32 = 2.0 / 3.0;
const MOST_DB: f32 = 6.0;
/// How far under the speech level the voice still counts as speaking: 20 dB.
const SPEAKING: f32 = 0.01;
const MEASURE_MS: f32 = 5.0;
const ATTACK_MS: f32 = 1.0;
const RELEASE_MS: f32 = 40.0;

pub(super) struct Deesser {
  split: Split,
  /// The energy below which nobody is speaking.
  quiet: f32,
  measure: f32,
  attack: f32,
  release: f32,
  high: Vec<f32>,
}

/// The step a one-pole follower with time constant `ms` takes per sample.
fn step(rate: u32, ms: f32) -> f32 {
  1.0 - (-1_000.0 / (ms * rate as f32)).exp()
}

impl Deesser {
  /// A de-esser for a voice whose speech has an average energy of `speech`.
  pub(super) fn new(rate: u32, speech: f32) -> Self {
    Self {
      split: Split::new(rate, SPLIT_HZ),
      quiet: speech * SPEAKING,
      measure: step(rate, MEASURE_MS),
      attack: step(rate, ATTACK_MS),
      release: step(rate, RELEASE_MS),
      high: Vec::new(),
    }
  }

  /// Softens the sibilants in `samples`, starting from rest: a run should
  /// begin a little before the samples that are kept.
  pub(super) fn apply(&mut self, samples: &mut [f32]) {
    self.split.high(samples, &mut self.high);
    let (mut whole, mut upper, mut cut) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (sample, &high) in samples.iter_mut().zip(&self.high) {
      whole += self.measure * (*sample * *sample - whole);
      upper += self.measure * (high * high - upper);
      let lower = (whole - upper).max(f32::MIN_POSITIVE);
      let target = if whole > self.quiet && upper > lower * BALANCE {
        (10.0 * (upper / (lower * BALANCE)).log10() * SHARE).min(MOST_DB)
      } else {
        0.0
      };
      let follow = if target > cut {
        self.attack
      } else {
        self.release
      };
      cut += follow * (target - cut);
      if cut > 0.01 {
        *sample -= (1.0 - 10_f32.powf(-cut / 20.0)) * high;
      }
    }
  }
}
