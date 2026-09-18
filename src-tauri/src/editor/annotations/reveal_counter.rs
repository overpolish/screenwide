// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reveal a timed counter arrives and leaves through.
//!
//! A counter has no path to be drawn along, so the arrow's arc-length window
//! means nothing to it: it is a disc with a number in it, and the only way a
//! disc can arrive is to grow into place. It scales up as it fades in and
//! shrinks a little as it fades out, which reads as the number being put down
//! and picked back up.
//!
//! It is also much quicker than an arrow's. An arrow's drawing is the thing
//! the viewer is meant to follow, so it is given a second; a counter is read
//! the instant it lands, and a slow one only delays the reading.
//!
//! The window - `low` and `high` - stays whole throughout. Only `scale` and
//! `opacity` move, and the compositor applies both to the disc, its tail and
//! its number together.

use super::AnnotationReveal;
use crate::editor::effect_animation::{ease_in_out_cubic, ease_out_cubic};

/// How long a counter takes to arrive. The twin of
/// `ANNOTATION_COUNTER_DRAW_IN_MS` in `src/features/editor/annotations.ts`,
/// which places a fresh clip this far before the playhead so the counter is
/// whole by the time the playhead is reached.
pub(crate) const COUNTER_REVEAL_IN_MS: f32 = 320.0;

/// How long it takes to leave. Longer than its arrival: an arrival is a thing
/// appearing, which the eye catches however quick it is, while a departure that
/// is quicker than the eye reads as the annotation being cut rather than
/// leaving. Still well short of an arrow's, which has a stroke to undraw.
pub(crate) const COUNTER_REVEAL_OUT_MS: f32 = 420.0;

/// The most of a clip either phase may take, so a clip shorter than the two
/// phases together still arrives before it starts leaving.
const COUNTER_PHASE_SHARE: f32 = 1.0 / 3.0;

/// The size a counter starts from and shrinks back to. Growing from nothing
/// spends the first frames on a speck of a disc with an unreadable number in
/// it; starting a little over half size, the number is legible from the first
/// frame it is solid enough to read.
const COUNTER_REVEAL_FROM: f32 = 0.55;

/// The reveal a counter's clip is at, `elapsed_ms` into a clip lasting
/// `duration_ms`. Everything is derived from the clip's bounds and the frame's
/// source time, so a scrub backwards lands on exactly the frame playing
/// forwards drew.
///
/// Size and opacity follow one curve rather than two of their own: a disc that
/// grew on one schedule and faded on another reads as two things happening.
/// Arriving eases out, the way the hover halo's pulse does - quick off the
/// annotation, settling gently - and leaving eases in and out over the whole of
/// its phase, so the last frames are a fade rather than a cut.
///
/// `frame_ms` is the exposure interval in source time, and the state one
/// interval back rides along as `previous`: a disc that grows through a frame
/// covered every size in between, so the compositor smears it over them exactly
/// as it smears a travelling arrow. A paused preview passes zero and the
/// shutter starts where it ends.
pub(crate) fn counter_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  frame_ms: f32,
) -> AnnotationReveal {
  let opening_ms = COUNTER_REVEAL_IN_MS.min(duration_ms * COUNTER_PHASE_SHARE);
  let closing_ms = COUNTER_REVEAL_OUT_MS.min(duration_ms * COUNTER_PHASE_SHARE);
  if !opening_ms.is_finite() || opening_ms <= 0.0 || !elapsed_ms.is_finite() {
    return AnnotationReveal::WHOLE;
  }
  // One formula for size and opacity at a source time, so the current frame
  // and the one before it - the blur's lead - share it.
  let presence = |at_ms: f32| {
    let arriving = ease_out_cubic((at_ms / opening_ms).clamp(0.0, 1.0));
    let leaving =
      ease_in_out_cubic(((at_ms - (duration_ms - closing_ms)) / closing_ms).clamp(0.0, 1.0));
    // One phase at a time: the clip caps each to a third of itself, so the
    // phase that is not running sits at rest on its own extreme.
    arriving * (1.0 - leaving)
  };
  let scale = |presence: f32| COUNTER_REVEAL_FROM + (1.0 - COUNTER_REVEAL_FROM) * presence;
  let now = presence(elapsed_ms);
  let before = presence(elapsed_ms - frame_ms.max(0.0));
  AnnotationReveal {
    low: 0.0,
    high: 1.0,
    scale: scale(now),
    opacity: now,
    previous: [0.0, 1.0, scale(before), before],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const CLIP_MS: f32 = 2_000.0;

  #[test]
  fn a_counter_grows_into_place_and_shrinks_away() {
    let at = |elapsed| counter_reveal_window(elapsed, CLIP_MS, 0.0);
    assert_eq!(at(0.0).scale, COUNTER_REVEAL_FROM);
    assert_eq!(at(0.0).opacity, 0.0);
    // Whole by the end of its arrival, and still whole in the middle.
    assert_eq!(at(COUNTER_REVEAL_IN_MS).scale, 1.0);
    assert!(at(1_000.0).is_whole());
    // Arriving eases out: half way through the phase it is most of the way
    // there, and size and opacity are on the same curve.
    let half = at(COUNTER_REVEAL_IN_MS / 2.0);
    assert!(
      half.opacity > 0.8,
      "{} is not most of the way",
      half.opacity
    );
    assert!(
      (half.scale - (COUNTER_REVEAL_FROM + (1.0 - COUNTER_REVEAL_FROM) * half.opacity)).abs()
        < 1e-6
    );
    // And gone by the end of the clip.
    assert_eq!(at(CLIP_MS).opacity, 0.0);
    assert!(at(CLIP_MS).scale < 1.0);
  }

  #[test]
  fn leaving_takes_its_whole_phase_rather_than_cutting_out() {
    let at = |elapsed| counter_reveal_window(elapsed, CLIP_MS, 0.0);
    let leaving_starts = CLIP_MS - COUNTER_REVEAL_OUT_MS;
    assert_eq!(at(leaving_starts).opacity, 1.0);
    // Still mostly there a quarter of the way out, and mostly gone three
    // quarters of the way: an ease-in-out spends its speed in the middle
    // rather than dropping at either end.
    let quarter = at(leaving_starts + COUNTER_REVEAL_OUT_MS * 0.25).opacity;
    let three_quarters = at(leaving_starts + COUNTER_REVEAL_OUT_MS * 0.75).opacity;
    assert!(quarter > 0.85, "{quarter} left too fast");
    assert!(three_quarters < 0.15, "{three_quarters} lingered");
    assert!(at(leaving_starts + COUNTER_REVEAL_OUT_MS * 0.5).opacity > 0.4);
  }

  #[test]
  fn the_window_stays_whole_so_nothing_is_drawn_along() {
    for elapsed in [0.0, 100.0, 1_000.0, CLIP_MS] {
      let window = counter_reveal_window(elapsed, CLIP_MS, 0.0);
      assert_eq!((window.low, window.high), (0.0, 1.0));
      // A paused preview has no interval to smear over, so the shutter
      // starts where it ends.
      assert_eq!(window.previous, [0.0, 1.0, window.scale, window.opacity]);
    }
  }

  #[test]
  fn a_clip_shorter_than_both_phases_still_arrives_before_it_leaves() {
    // A 300ms clip caps each phase to 100ms, so it is whole in between.
    let at = |elapsed| counter_reveal_window(elapsed, 300.0, 0.0);
    assert_eq!(at(100.0).scale, 1.0);
    assert_eq!(at(150.0).opacity, 1.0);
    assert!(at(300.0).scale < 1.0);
  }
}
