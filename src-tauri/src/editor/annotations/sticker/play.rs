// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a sticker showing an animated picture - a GIF, an animated PNG or
//! WebP - plays, and which of its frames a moment shows.
//!
//! A still shows the frame chosen for it. A recording plays the animation in
//! recording time from the start of the sticker's clip, starting on that same
//! frame: it loops for as long as the clip lasts, or plays through once, in
//! which case the clip lasts exactly one run.

use serde::{Deserialize, Serialize};

/// How a sticker's animation plays. Absent from a sticker whose picture does
/// not move. The twin of `StickerPlay` in
/// `src/features/editor/annotations/annotations.ts`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerPlay {
  /// How long one run of the animation lasts, in milliseconds.
  pub cycle_ms: f64,
  /// How many frames one run has.
  pub frames: u32,
  /// The frame a still shows, counted from zero, and the one playback
  /// starts on.
  #[serde(default)]
  pub frame: u32,
  /// Whether it plays through once rather than looping; its clip then lasts
  /// exactly one run.
  #[serde(default)]
  pub once: bool,
  /// How far into its clip the frame being drawn is, in milliseconds; absent
  /// on a still. Derived from the clip and the frame's source time every
  /// frame and never stored, as an annotation's reveal is.
  #[serde(skip)]
  pub clock_ms: Option<f64>,
}

impl StickerPlay {
  /// Whether the play describes an animation that can be drawn: a document
  /// read from disk carries whatever it was written with.
  pub(crate) fn is_usable(&self) -> bool {
    self.frames > 1 && self.cycle_ms.is_finite() && self.cycle_ms > 0.0
  }
}

/// Sets how far into its clip `annotation` is drawn, where it is a sticker
/// whose picture moves; anything else is left as it is.
pub(crate) fn clocked(annotation: &mut crate::editor::annotations::Annotation, clock_ms: f64) {
  if let crate::editor::annotations::AnnotationShape::Sticker {
    play: Some(play), ..
  } = &mut annotation.shape
  {
    play.clock_ms = Some(clock_ms);
  }
}

/// Which frame shows, out of frames that start `starts` milliseconds into a
/// run `cycle_ms` long: on a still (`clock_ms` absent), frame `start`; while
/// playing, the frame `clock_ms` after `start` began, wrapping round the run.
/// Played `once`, the last moment of the run shows the frame before `start`
/// rather than wrapping back to it.
pub(crate) fn frame_at(
  starts: &[u32],
  cycle_ms: u32,
  start: u32,
  clock_ms: Option<f32>,
  once: bool,
) -> usize {
  let last = starts.len().saturating_sub(1);
  let start = (start as usize).min(last);
  let Some(clock) = clock_ms.filter(|clock| clock.is_finite() && *clock > 0.0) else {
    return start;
  };
  if cycle_ms == 0 {
    return start;
  }
  let elapsed = clock as u64;
  let elapsed = if once {
    elapsed.min(u64::from(cycle_ms) - 1)
  } else {
    elapsed
  };
  let at = (u64::from(starts[start]) + elapsed) % u64::from(cycle_ms);
  // The frame the moment falls in: the last whose start is at or before it.
  starts.partition_point(|begins| u64::from(*begins) <= at) - 1
}

#[cfg(test)]
mod tests {
  use super::*;

  // Three frames of 100, 200 and 100 milliseconds.
  const STARTS: [u32; 3] = [0, 100, 300];
  const CYCLE: u32 = 400;

  #[test]
  fn a_still_shows_its_chosen_frame() {
    assert_eq!(frame_at(&STARTS, CYCLE, 1, None, false), 1);
    assert_eq!(frame_at(&STARTS, CYCLE, 9, None, false), 2);
  }

  #[test]
  fn playback_starts_on_the_chosen_frame_and_loops() {
    assert_eq!(frame_at(&STARTS, CYCLE, 0, Some(150.0), false), 1);
    assert_eq!(frame_at(&STARTS, CYCLE, 0, Some(399.0), false), 2);
    assert_eq!(frame_at(&STARTS, CYCLE, 0, Some(410.0), false), 0);
    // From the second frame, 250 ms on is 350 into the run: the third.
    assert_eq!(frame_at(&STARTS, CYCLE, 1, Some(250.0), false), 2);
    // And 320 ms on wraps to 20 into the next run: the first.
    assert_eq!(frame_at(&STARTS, CYCLE, 1, Some(320.0), false), 0);
  }

  #[test]
  fn played_once_it_rests_on_the_frame_before_the_one_it_started_on() {
    assert_eq!(frame_at(&STARTS, CYCLE, 0, Some(5_000.0), true), 2);
    assert_eq!(frame_at(&STARTS, CYCLE, 1, Some(5_000.0), true), 0);
  }
}
