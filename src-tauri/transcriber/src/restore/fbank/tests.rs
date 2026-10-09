// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::f32::consts::TAU;

use super::{Fbank, WIDTH};

fn tone(seconds: f32) -> Vec<f32> {
  (0..(seconds * 16_000.0) as usize)
    .map(|at| 0.3 * (TAU * 220.0 * at as f32 / 16_000.0).sin())
    .collect()
}

#[test]
fn pairs_frames_of_25_ms_every_10_ms() {
  let (features, rows) = Fbank::new(16_000).features(&tone(3.0));
  // 1 + (48000 - 400) / 160 frames, paired.
  assert_eq!(rows, 149);
  assert_eq!(features.len(), rows * WIDTH);
}

#[test]
fn normalises_each_band() {
  let (features, rows) = Fbank::new(16_000).features(&tone(3.0));
  for band in 0..WIDTH {
    let values: Vec<f32> = (0..rows).map(|row| features[row * WIDTH + band]).collect();
    let mean = values.iter().sum::<f32>() / rows as f32;
    assert!(mean.abs() < 0.2, "band {band} mean {mean}");
  }
}

/// Values Hugging Face's `SeamlessM4TFeatureExtractor` gives for the same
/// samples: a 220 Hz tone, joined by one at 3.1 kHz after half a second.
#[test]
fn matches_the_extractor_the_model_was_trained_with() {
  let samples: Vec<f32> = (0..32_000)
    .map(|at| {
      let time = at as f32 / 16_000.0;
      let high = if at > 8_000 {
        0.1 * (TAU * 3_100.0 * time).sin()
      } else {
        0.0
      };
      0.3 * (TAU * 220.0 * time).sin() + high
    })
    .collect();
  let (features, rows) = Fbank::new(16_000).features(&samples);
  assert_eq!(rows, 99);
  let expected: [(usize, usize, [f32; 6]); 3] = [
    (10, 0, [0.8514, 0.7251, -0.7199, -1.2805, -1.0321, 0.4138]),
    (60, 20, [-0.0423, 0.1594, 0.3113, -0.0169, -0.1285, 0.7670]),
    (90, 100, [0.3039, 0.3277, -0.3103, 0.1233, 0.3723, -0.3628]),
  ];
  for (row, start, values) in expected {
    for (offset, value) in values.into_iter().enumerate() {
      let got = features[row * WIDTH + start + offset];
      assert!(
        (got - value).abs() < 5e-3,
        "row {row}, value {}: {got} against {value}",
        start + offset
      );
    }
  }
}
