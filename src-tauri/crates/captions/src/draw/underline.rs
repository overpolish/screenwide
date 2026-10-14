// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The line under the word being said: as each word is said, it slides and
//! stretches from under the word before to under it, eased out, as long as
//! the two share a line; to a word on another line it moves at once.

use super::caption::DrawnWord;
use super::place::MIDDLE_ABOVE_BASELINE;
use crate::motion::CaptionEasing;

/// How long the line takes to reach the next word.
const SLIDE_MS: f64 = 150.0;
/// How far below its word's baseline the line's middle sits, and how thick
/// it is, both in ems of the word's type.
const BELOW_BASELINE: f64 = 0.17;
const THICKNESS: f64 = 0.08;

/// Where the line sits under `word` with its middle at `centre`: its own
/// middle, and how wide and thick it is.
fn under(word: &DrawnWord, centre: [f64; 2]) -> ([f64; 2], [f64; 2]) {
  let size = word.placed.size;
  (
    [
      centre[0],
      centre[1] + (MIDDLE_ABOVE_BASELINE + BELOW_BASELINE) * size,
    ],
    [word.placed.width, THICKNESS * size],
  )
}

/// The line under word `said` of `words` at `at_ms`: its middle then and as
/// the frame's exposure began at `opened_ms`, and how wide and thick it is,
/// or none where that word is not drawn.
pub(super) fn underline(
  words: &[DrawnWord],
  said: usize,
  at_ms: f64,
  opened_ms: f64,
) -> Option<([[f64; 2]; 2], [f64; 2])> {
  let to = words.iter().find(|word| word.index == said)?;
  let from = said
    .checked_sub(1)
    .and_then(|before| words.iter().find(|word| word.index == before))
    .filter(|from| (from.placed.line[0] - to.placed.line[0]).abs() < 0.5);
  let at = |ms: f64, centre: fn(&DrawnWord) -> [f64; 2]| {
    let end = under(to, centre(to));
    let Some(from) = from else {
      return end;
    };
    let start = under(from, centre(from));
    let progress =
      CaptionEasing::EaseOut.eased(((ms - to.word.start_ms) / SLIDE_MS).clamp(0.0, 1.0));
    let lerp = |from: [f64; 2], to: [f64; 2]| {
      [0, 1].map(|axis| from[axis] + (to[axis] - from[axis]) * progress)
    };
    (lerp(start.0, end.0), lerp(start.1, end.1))
  };
  let (now, block) = at(at_ms, |word| word.placed.centre);
  let (opened, _) = at(opened_ms, |word| word.previous);
  Some(([now, opened], block))
}
