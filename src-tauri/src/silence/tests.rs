// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::is_silent;

const RATE: u32 = 16_000;

/// A 440 Hz tone whose RMS level is `dbfs`, lasting `ms`.
fn tone(dbfs: f32, ms: u32) -> Vec<f32> {
  let amplitude = 10f32.powf(dbfs / 20.0) * std::f32::consts::SQRT_2;
  (0..RATE * ms / 1_000)
    .map(|index| amplitude * (std::f32::consts::TAU * 440.0 * index as f32 / RATE as f32).sin())
    .collect()
}

#[test]
fn nothing_and_a_hum_under_the_threshold_are_silent() {
  assert!(is_silent(&[], RATE));
  assert!(is_silent(&vec![0.0; RATE as usize], RATE));
  assert!(is_silent(&tone(-55.0, 2_000), RATE));
}

#[test]
fn a_short_quiet_word_in_a_silent_note_is_sound() {
  let mut note = tone(-55.0, 1_000);
  note.extend(tone(-40.0, 60));
  note.extend(tone(-55.0, 1_000));
  assert!(!is_silent(&note, RATE));
}
