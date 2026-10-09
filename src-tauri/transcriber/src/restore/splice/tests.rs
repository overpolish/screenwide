// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::splice;

const OVERLAP: usize = 96_000;

/// Two takes on the same voice that do not line up: the same tone half a
/// cycle apart, silent over `pause`.
fn takes(pause: std::ops::Range<usize>) -> (Vec<f32>, Vec<f32>) {
  let tone = |at: usize, phase: f32| {
    if pause.contains(&at) {
      0.0
    } else {
      #[expect(clippy::cast_precision_loss, reason = "test lengths")]
      (at as f32 * 0.05 + phase).sin()
    }
  };
  let held = (0..OVERLAP).map(|at| tone(at, 0.0)).collect();
  let voice = (0..OVERLAP + 48_000)
    .map(|at| tone(at, std::f32::consts::PI))
    .collect();
  (held, voice)
}

#[test]
fn the_cut_falls_in_a_pause_and_never_blends_the_takes_elsewhere() {
  let pause = 70_000..73_000;
  let (held, mut voice) = takes(pause.clone());
  let fresh = voice.clone();
  splice(&held, &mut voice);
  assert_eq!(voice[..pause.start], held[..pause.start]);
  assert_eq!(voice[pause.end..], fresh[pause.end..]);
}

#[test]
fn a_pause_while_the_new_chunk_is_still_settling_is_passed_over() {
  let (held, mut voice) = takes(10_000..13_000);
  splice(&held, &mut voice);
  assert_eq!(voice[..OVERLAP / 2], held[..OVERLAP / 2]);
}
