// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const RATE: usize = 8_000;

/// A steady noise, not random, so the tests are repeatable.
fn noise(length: usize, level: f32) -> Vec<f32> {
  let mut state = 0x2545_f491_u32;
  (0..length)
    .map(|_| {
      state ^= state << 13;
      state ^= state >> 17;
      state ^= state << 5;
      #[expect(clippy::cast_precision_loss, reason = "a test signal")]
      let unit = state as f32 / u32::MAX as f32;
      (unit * 2.0 - 1.0) * level
    })
    .collect()
}

/// Drum hits every `period_ms` from `first_ms`, over quiet noise.
fn hits(seconds: usize, period_ms: f64, first_ms: f64) -> Vec<f32> {
  let mut samples = noise(seconds * RATE, 0.01);
  let mut at = first_ms;
  #[expect(clippy::cast_precision_loss, reason = "a test signal")]
  while at < (seconds * 1_000) as f64 {
    #[expect(
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss,
      reason = "a sample index"
    )]
    let start = (at * 8.0) as usize;
    for (index, burst) in noise(800, 0.8).iter().enumerate() {
      #[expect(clippy::cast_precision_loss, reason = "a decay")]
      let decay = (-(index as f32) / 120.0).exp();
      if let Some(sample) = samples.get_mut(start + index) {
        *sample += burst * decay;
      }
    }
    at += period_ms;
  }
  samples
}

#[test]
fn finds_the_tempo_and_where_the_beats_fall() {
  let beat = analyse(&hits(60, 60_000.0 / 128.0, 230.0), RATE)
    .beat
    .expect("a steady beat");
  assert!((60_000.0 / beat.period_ms - 128.0).abs() < 0.05, "{beat:?}");
  assert!((beat.nearest(230.0) - 230.0).abs() < 10.0, "{beat:?}");
  // The hundredth beat, where a tempo a little off would have drifted.
  assert!((beat.nearest(47_105.0) - 47_105.0).abs() < 10.0, "{beat:?}");
}

#[test]
fn noise_has_no_steady_beat() {
  assert_eq!(analyse(&noise(60 * RATE, 0.3), RATE).beat, None);
}
