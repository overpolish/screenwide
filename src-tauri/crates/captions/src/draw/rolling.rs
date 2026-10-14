// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption whose words roll through on one line, a few at a time: each
//! comes on as it is said, and once the line holds as many as the style
//! shows, the oldest goes off as the next comes on. The line slides as one
//! to make room, the word going off carried along with it, so no word ever
//! runs into another.

use std::ops::Range;

use super::caption::DrawnWord;
use super::place::{place_lines, Placed, Placement};
use crate::motion::CaptionEasing;
use crate::style::CaptionStyle;
use crate::{type_size_px, Caption, CaptionFrame, CaptionWord, Measure};

/// `placed` moved by `by`.
fn moved(placed: Placed, by: [f64; 2]) -> Placed {
  Placed {
    centre: [placed.centre[0] + by[0], placed.centre[1] + by[1]],
    ..placed
  }
}

/// The words of `caption` drawn `at_ms` into the edited video, rolling
/// through `visible` at a time, set as `style` sets them, those still on
/// the line going off with the caption from `exits_ms`, with the caption's
/// own type size. Words going off come first, so those staying are drawn
/// over them.
pub(super) fn rolling_words<'a>(
  caption: &'a Caption,
  style: &CaptionStyle,
  visible: usize,
  (frame, at_ms, exits_ms): (CaptionFrame, f64, f64),
  measure: &mut Measure,
) -> (Vec<DrawnWord<'a>>, f64) {
  let words: Vec<&CaptionWord> = caption.lines.iter().flatten().collect();
  let said = words.partition_point(|word| word.start_ms <= at_ms);
  // The words on the line once `end` of them have been said.
  let window = |end: usize| end.saturating_sub(visible)..end;
  let mut place = |range: Range<usize>| -> Placement {
    place_lines(
      [words[range].iter().copied()],
      caption.spaced,
      style,
      frame,
      measure,
    )
  };
  let slide_ms = style.motion.word_entrance_ms();
  // As word `next` comes on, the line before it, how far the line moves to
  // make room, and how much of that way it has come, eased out over as
  // long as a word takes to come on.
  let mut slide = |next: usize| {
    let before = place(window(next));
    let after = place(window(next + 1));
    // The word said just before it is on the line both before and after.
    let by = next.checked_sub(1).map_or([0.0; 2], |kept| {
      let from = before.words[kept - window(next).start].centre;
      let to = after.words[kept - window(next + 1).start].centre;
      [to[0] - from[0], to[1] - from[1]]
    });
    let progress = if slide_ms > 0.0 {
      CaptionEasing::EaseOut.eased(((at_ms - words[next].start_ms) / slide_ms).clamp(0.0, 1.0))
    } else {
      1.0
    };
    (before, after, by, progress)
  };
  let Some(latest) = said.checked_sub(1) else {
    return (Vec::new(), type_size_px(style, frame));
  };
  let mut drawn = Vec::new();
  // A word goes off as the one `visible` after it comes on, carried along
  // the way the line slides then, until its exit is over.
  let exit_ms = style.motion.word_exit_ms();
  for index in 0..window(said).start {
    let pushed_by = index + visible;
    let leaves_ms = words[pushed_by].start_ms;
    if at_ms >= leaves_ms + exit_ms {
      continue;
    }
    let (before, _, by, progress) = slide(pushed_by);
    drawn.push(DrawnWord {
      word: words[index],
      index,
      placed: moved(before.words[0], by.map(|by| by * progress)),
      leaves_ms,
    });
  }
  // The rest slide as one, the word just said coming on at the line's end.
  let (_, now, by, progress) = slide(latest);
  for (index, &placed) in window(said).zip(&now.words) {
    drawn.push(DrawnWord {
      word: words[index],
      index,
      placed: moved(placed, by.map(|by| -by * (1.0 - progress))),
      leaves_ms: exits_ms,
    });
  }
  (drawn, type_size_px(style, frame))
}
