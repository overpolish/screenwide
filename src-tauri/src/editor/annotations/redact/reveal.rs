// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ramp an animated redaction arrives through.
//!
//! A redaction has no path to draw and no disc to grow. It arrives by
//! covering more and more of what is under it: a fill fades in, and a
//! classic pixelation's blocks or a blur's cells grow from a single pixel to
//! their full size. It takes the counter's quick timing, and it never
//! leaves: the end of a clip is where what it hides may show again, so it
//! stays whole to its last frame. Until it has arrived, what it covers shows
//! through, which is why a redaction starts without the ramp.
//!
//! `opacity` and `scale` both carry how far it has arrived, and the
//! compositor reads it from `opacity`; the window stays whole.

use crate::editor::annotations::counter::reveal::COUNTER_REVEAL_IN_MS;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::effect_animation::ease_out_cubic;

/// The most of a clip the ramp may take, so a short clip is covered whole
/// for most of it.
const REDACT_PHASE_SHARE: f32 = 1.0 / 3.0;

/// The reveal a redaction's clip is at, `elapsed_ms` into a clip lasting
/// `duration_ms`. `frame_ms` is the exposure interval, whose start rides
/// along as `previous` the way every kind's does.
pub(crate) fn redact_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  frame_ms: f32,
) -> AnnotationReveal {
  let opening_ms = COUNTER_REVEAL_IN_MS.min(duration_ms * REDACT_PHASE_SHARE);
  if !opening_ms.is_finite() || opening_ms <= 0.0 || !elapsed_ms.is_finite() {
    return AnnotationReveal::WHOLE;
  }
  let arrived = |at_ms: f32| ease_out_cubic((at_ms / opening_ms).clamp(0.0, 1.0));
  let now = arrived(elapsed_ms);
  let before = arrived(elapsed_ms - frame_ms.max(0.0));
  AnnotationReveal {
    low: 0.0,
    high: 1.0,
    scale: now,
    opacity: now,
    previous: [0.0, 1.0, before, before],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_redaction_arrives_over_the_counters_time_and_never_leaves() {
    assert_eq!(redact_reveal_window(0.0, 3_000.0, 0.0).opacity, 0.0);
    let half = redact_reveal_window(COUNTER_REVEAL_IN_MS / 2.0, 3_000.0, 0.0).opacity;
    assert!(half > 0.5 && half < 1.0, "{half}");
    assert!(redact_reveal_window(COUNTER_REVEAL_IN_MS, 3_000.0, 0.0).is_whole());
    // The last frame of the clip is still covered whole.
    assert!(redact_reveal_window(2_999.0, 3_000.0, 16.0).is_whole());
  }

  #[test]
  fn a_short_clip_is_covered_whole_for_most_of_it() {
    assert!(redact_reveal_window(100.0, 300.0, 0.0).is_whole());
  }
}
