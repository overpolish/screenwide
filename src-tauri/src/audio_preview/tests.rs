// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::LevelAccumulator;
use cpal::{BufferSize, StreamConfig};

fn accumulator(samples_per_update: u32) -> LevelAccumulator {
  LevelAccumulator::new(&StreamConfig {
    buffer_size: BufferSize::Default,
    channels: 1,
    sample_rate: samples_per_update * 30,
  })
}

#[test]
fn emits_at_thirty_frames_per_second() {
  let mut level = accumulator(4);
  let mut output = Vec::new();
  level.push([0.5, 0.5, 0.5], |value| output.push(value));
  assert!(output.is_empty());
  level.push([0.5], |value| output.push(value));
  assert_eq!(output.first().map(|value| value.round()), Some(-6.0));
}

#[test]
fn short_transients_reach_the_sample_peak() {
  let mut level = accumulator(4);
  let mut output = Vec::new();
  level.push([1.0, 0.0, 0.0, 0.0], |value| output.push(value));
  assert_eq!(output, [0.0]);
}

#[test]
fn resets_after_emitting_a_level() {
  let mut level = accumulator(2);
  let mut output = Vec::new();
  level.push([1.0, 1.0], |value| output.push(value));
  output.clear();
  level.push([0.1, 0.1], |value| output.push(value));
  assert_eq!(output.first().map(|value| value.round()), Some(-20.0));
}

#[test]
fn preserves_samples_beyond_an_update_boundary() {
  let mut level = accumulator(2);
  let mut output = Vec::new();
  level.push([1.0, 1.0, 0.1, 0.1], |value| output.push(value));
  assert_eq!(output.len(), 2);
  assert_eq!(output[0], 0.0);
  assert_eq!(output[1].round(), -20.0);
}

#[test]
fn clamps_silence_to_a_finite_floor() {
  let mut level = accumulator(1);
  let mut output = Vec::new();
  level.push([0.0], |value| output.push(value));
  assert_eq!(output, [-160.0]);
}
