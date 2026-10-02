// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a spotlight fades in and out over its clip.
//!
//! A spotlight has no path to draw and nothing to grow. The shade deepens
//! around it as it arrives and lifts again as it leaves, which reads as the
//! lights going down and coming back up. It is slower than a counter's
//! arrival - a change in the whole picture's light is jarring when quick -
//! and leaves slower still, for the same reason a counter's departure is.
//!
//! The window stays whole; only `opacity` moves, and the passes read the
//! spotlight's presence from it. `previous` holds the same presence rather
//! than the frame before's: a fade is the same whether the shutter is open
//! or not, and there is nothing to smear. `scale`, which a spotlight has no
//! size to apply to, carries how far its blur has arrived beside its light:
//! all of it, except while the light glides between a spotlight that blurs
//! and one that does not.
//!
//! Where one spotlight hands its light to the next, `handoff` joins them:
//! the one handing on does not fade out and the one taking over does not
//! fade in, so the shade holds through the join.

use crate::editor::annotations::reveal::{AnnotationReveal, REVEAL_PHASE_SHARE};
use crate::editor::effect_animation::{ease_in_out_cubic, ease_out_cubic};

/// How long a spotlight takes to arrive. The twin of
/// `ANNOTATION_SPOTLIGHT_DRAW_IN_MS` in `src/features/editor/annotations/annotation-kinds.ts`,
/// which places a fresh clip this far before the playhead so the light is
/// whole by the time the playhead is reached.
pub(crate) const SPOTLIGHT_FADE_IN_MS: f32 = 400.0;

/// How long it takes to leave.
pub(crate) const SPOTLIGHT_FADE_OUT_MS: f32 = 500.0;

/// Which ends of a clip its light is handed over at: `from` where it takes
/// the light from the spotlight before, `onward` where it hands it to the
/// next. Neither end fades.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpotlightJoins {
  pub(crate) from: bool,
  pub(crate) onward: bool,
}

/// The reveal a spotlight's clip is at, `elapsed_ms` into a clip lasting
/// `duration_ms`, arriving and leaving except at its `joins`, with
/// `blur_share` of its blur arrived.
pub(crate) fn spotlight_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  joins: SpotlightJoins,
  blur_share: f32,
) -> AnnotationReveal {
  let blur_share = if blur_share.is_finite() {
    blur_share.clamp(0.0, 1.0)
  } else {
    1.0
  };
  let opening_ms = SPOTLIGHT_FADE_IN_MS.min(duration_ms * REVEAL_PHASE_SHARE);
  let closing_ms = SPOTLIGHT_FADE_OUT_MS.min(duration_ms * REVEAL_PHASE_SHARE);
  let now = if !opening_ms.is_finite() || opening_ms <= 0.0 || !elapsed_ms.is_finite() {
    1.0
  } else {
    // Arriving eases out and leaving eases in and out, one phase at a time,
    // as a counter's presence does.
    let arriving = if joins.from {
      1.0
    } else {
      ease_out_cubic((elapsed_ms / opening_ms).clamp(0.0, 1.0))
    };
    let leaving = if joins.onward {
      0.0
    } else {
      ease_in_out_cubic(((elapsed_ms - (duration_ms - closing_ms)) / closing_ms).clamp(0.0, 1.0))
    };
    arriving * (1.0 - leaving)
  };
  AnnotationReveal {
    low: 0.0,
    high: 1.0,
    scale: blur_share,
    opacity: now,
    previous: [0.0, 1.0, blur_share, now],
  }
}
