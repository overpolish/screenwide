// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::grid_index;

/// An output frame's time as the export loop hands it over through an edit:
/// the frame's own time in whole microseconds, then mapped onto the source at
/// `rate`, rounded again.
fn through_an_edit(frame: u64, rate: f64) -> u64 {
  let output_us = (frame as f64 * 1_000_000.0 / 60.0).round();
  (output_us * rate).round() as u64
}

#[test]
fn every_exported_frame_takes_its_own_grid_frame() {
  for frame in 0..6_000 {
    assert_eq!(
      grid_index(through_an_edit(frame, 1.0)),
      frame as usize,
      "frame {frame}"
    );
  }
}

#[test]
fn a_doubled_speed_steps_two_grid_frames_a_frame() {
  for frame in 0..6_000 {
    assert_eq!(
      grid_index(through_an_edit(frame, 2.0)),
      2 * frame as usize,
      "frame {frame}"
    );
  }
}

#[test]
fn a_time_between_grid_frames_keeps_the_earlier_one() {
  assert_eq!(grid_index(25_000), 1);
  assert_eq!(grid_index(33_330), 1);
}
