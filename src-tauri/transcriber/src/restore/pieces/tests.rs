// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A tone with a short silence every `every` samples, the quiet moments a
/// piece should end in; one falls in every search for one.
fn speech_with_pauses(length: usize, every: usize) -> Vec<f32> {
  (0..length)
    .map(|at| {
      if at % every < BLOCK * 3 {
        0.0
      } else {
        (at as f32 * 0.05).sin() * 0.5
      }
    })
    .collect()
}

#[test]
fn pieces_cover_each_stretch_exactly_and_end_in_its_quiet_moments() {
  let samples = speech_with_pauses(120 * RATE, 9 * RATE / 10);
  let kept = Kept::new(
    &samples,
    &[[0, 50 * RATE as u64], [70 * RATE as u64, 120 * RATE as u64]],
  );
  let pieces = kept.pieces();
  for stretch in [0..50 * RATE, 70 * RATE..120 * RATE] {
    let mine: Vec<&Piece> = pieces
      .iter()
      .filter(|piece| piece.stretch == stretch)
      .collect();
    assert_eq!(mine.first().unwrap().samples.start, stretch.start);
    assert_eq!(mine.last().unwrap().samples.end, stretch.end);
    for pair in mine.windows(2) {
      assert_eq!(pair[0].samples.end, pair[1].samples.start);
      let cut = pair[0].samples.end;
      assert_eq!(samples[cut], 0.0, "a piece ends at {cut}, out of the quiet");
    }
    for piece in &mine {
      assert!(piece.samples.len() <= PIECE + PIECE / 2 + SEARCH);
    }
  }
}

#[test]
fn speech_heard_around_a_piece_skips_what_was_left_out() {
  let samples: Vec<f32> = (0..100).map(|at| at as f32).collect();
  let kept = Kept::new(&samples, &[[10, 20], [40, 50], [70, 80]]);
  assert_eq!(
    kept.before(45, 8),
    [15.0, 16.0, 17.0, 18.0, 19.0, 40.0, 41.0, 42.0, 43.0, 44.0][2..]
  );
  assert_eq!(
    kept.after(45, 8),
    [45.0, 46.0, 47.0, 48.0, 49.0, 70.0, 71.0, 72.0]
  );
  assert_eq!(kept.before(10, 5), Vec::<f32>::new());
}
