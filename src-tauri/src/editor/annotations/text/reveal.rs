// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reveal a timed text box arrives and leaves through.
//!
//! Two parts, one after the other: the box grows into place the way a counter
//! does, and only then does its pointer draw out of it to what it points at.
//! Leaving runs the other way round - the pointer draws back in, then the box
//! shrinks away - so a long pointer never sweeps in with the box around it.
//! `scale` and `opacity` carry the box, and `high` how far the pointer is
//! drawn out.

use crate::editor::annotations::counter::reveal::{
  counter_presence, counter_scale, COUNTER_REVEAL_IN_MS, COUNTER_REVEAL_OUT_MS,
};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::effect_animation::{ease_in_out_cubic, ease_out_cubic};

/// How long the pointer takes to draw out once the box is in, and to draw
/// back before it leaves.
const POINTER_IN_MS: f32 = 280.0;
const POINTER_OUT_MS: f32 = 240.0;

/// How long a text box takes to arrive, pointer and all. The twin of
/// `ANNOTATION_TEXT_DRAW_IN_MS` in `src/features/editor/annotation-kinds.ts`,
/// which places a fresh clip this far before the playhead so the box is whole
/// by the time the playhead is reached.
#[cfg(test)]
const TEXT_REVEAL_IN_MS: f32 = COUNTER_REVEAL_IN_MS + POINTER_IN_MS;

/// The most of a clip each end may take, so a clip shorter than both ends
/// together still arrives before it starts leaving.
const TEXT_PHASE_SHARE: f32 = 1.0 / 3.0;

/// The reveal a text box is at, `elapsed_ms` into a clip lasting
/// `duration_ms`, with the state one exposure interval back in `previous` so
/// a box that grows or a pointer that draws out through a frame is smeared
/// over what it covered.
pub(crate) fn text_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  frame_ms: f32,
) -> AnnotationReveal {
  if !elapsed_ms.is_finite() || !duration_ms.is_finite() || duration_ms <= 0.0 {
    return AnnotationReveal::WHOLE;
  }
  // A short clip shortens both parts of an end alike rather than dropping
  // the pointer's.
  let fit = |box_ms: f32, pointer_ms: f32| {
    let share = (duration_ms * TEXT_PHASE_SHARE / (box_ms + pointer_ms)).min(1.0);
    (box_ms * share, pointer_ms * share)
  };
  let (box_in, pointer_in) = fit(COUNTER_REVEAL_IN_MS, POINTER_IN_MS);
  let (box_out, pointer_out) = fit(COUNTER_REVEAL_OUT_MS, POINTER_OUT_MS);
  let at = |at_ms: f32| {
    let presence = counter_presence(at_ms, duration_ms, box_in, box_out);
    let drawing = ease_out_cubic(((at_ms - box_in) / pointer_in).clamp(0.0, 1.0));
    let withdrawn = ease_in_out_cubic(
      ((at_ms - (duration_ms - box_out - pointer_out)) / pointer_out).clamp(0.0, 1.0),
    );
    (presence, drawing * (1.0 - withdrawn))
  };
  let (now, drawn) = at(elapsed_ms);
  let (before, drawn_before) = at(elapsed_ms - frame_ms.max(0.0));
  AnnotationReveal {
    low: 0.0,
    high: drawn,
    scale: counter_scale(now),
    opacity: now,
    previous: [0.0, drawn_before, counter_scale(before), before],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const CLIP_MS: f32 = 4_000.0;

  #[test]
  fn the_box_arrives_before_its_pointer_draws_out() {
    let at = |elapsed| text_reveal_window(elapsed, CLIP_MS, 0.0);
    // Part way through the box's arrival the pointer has not started.
    let growing = at(COUNTER_REVEAL_IN_MS * 0.5);
    assert!(growing.scale < 1.0 && growing.high == 0.0, "{growing:?}");
    // With the box in, the pointer draws out, and then stands whole.
    let pointing = at(COUNTER_REVEAL_IN_MS + POINTER_IN_MS * 0.5);
    assert_eq!(pointing.scale, 1.0);
    assert!(pointing.high > 0.0 && pointing.high < 1.0, "{pointing:?}");
    assert_eq!(at(TEXT_REVEAL_IN_MS).high, 1.0);
  }

  #[test]
  fn the_pointer_draws_back_in_before_the_box_leaves() {
    let at = |elapsed| text_reveal_window(elapsed, CLIP_MS, 0.0);
    let withdrawing = at(CLIP_MS - COUNTER_REVEAL_OUT_MS - POINTER_OUT_MS * 0.5);
    assert_eq!(withdrawing.scale, 1.0);
    assert!(withdrawing.high > 0.0 && withdrawing.high < 1.0);
    let leaving = at(CLIP_MS - COUNTER_REVEAL_OUT_MS * 0.5);
    assert_eq!(leaving.high, 0.0);
    assert!(leaving.scale < 1.0);
  }

  #[test]
  fn a_short_clip_still_arrives_whole_before_it_leaves() {
    let short = 900.0;
    let whole = text_reveal_window(short * TEXT_PHASE_SHARE, short, 0.0);
    assert_eq!(whole.high, 1.0);
    assert_eq!(whole.scale, 1.0);
  }

  /// The editor reaches a fresh clip back by the whole arrival, and
  /// TypeScript cannot read this constant, so it keeps its own copy.
  #[test]
  fn the_editor_places_a_text_clip_by_this_arrival() {
    const SOURCE: &str = include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../src/features/editor/annotation-kinds.ts"
    ));
    let declared = SOURCE
      .lines()
      .find_map(|line| {
        line
          .trim()
          .strip_prefix("const ANNOTATION_TEXT_DRAW_IN_MS = ")
      })
      .and_then(|value| value.trim_end_matches(';').parse::<f32>().ok());
    assert_eq!(declared, Some(TEXT_REVEAL_IN_MS));
  }
}
