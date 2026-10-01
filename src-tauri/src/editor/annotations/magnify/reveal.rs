// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a magnifier arrives and leaves over its clip.
//!
//! The loupe comes out of its zoom area: it starts as the zoom area itself,
//! showing what it covers at its own size, and grows as it travels to its
//! place, so the enlargement arrives with it; the rim, the line and the
//! shadow fade in on the way. It leaves the same way back, into the zoom
//! area.
//!
//! The window stays whole. `scale` is how far the loupe has travelled and
//! `opacity` how present the rest is, on one curve, since a loupe that moved
//! on one schedule and faded on another reads as two things happening. The
//! state one exposure interval back rides along as `previous`, so the
//! compositor smears a loupe that travelled through a frame over its way.

use crate::editor::annotations::counter::reveal::counter_presence;
use crate::editor::annotations::reveal::AnnotationReveal;

/// How long a magnifier takes to arrive. The twin of
/// `ANNOTATION_MAGNIFY_DRAW_IN_MS` in `src/features/editor/annotations/annotation-kinds.ts`.
pub(crate) const MAGNIFY_REVEAL_IN_MS: f32 = 450.0;

/// How long it takes to go back into its zoom area.
pub(crate) const MAGNIFY_REVEAL_OUT_MS: f32 = 400.0;

/// The most of a clip either phase may take, so a short clip still arrives
/// before it leaves.
const MAGNIFY_PHASE_SHARE: f32 = 1.0 / 3.0;

/// The reveal a magnifier's clip is at, `elapsed_ms` into a clip lasting
/// `duration_ms`. `frame_ms` is the exposure interval in source time; a
/// paused preview passes zero, and the shutter starts where it ends.
pub(crate) fn magnify_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  frame_ms: f32,
) -> AnnotationReveal {
  let opening_ms = MAGNIFY_REVEAL_IN_MS.min(duration_ms * MAGNIFY_PHASE_SHARE);
  let closing_ms = MAGNIFY_REVEAL_OUT_MS.min(duration_ms * MAGNIFY_PHASE_SHARE);
  if !opening_ms.is_finite() || opening_ms <= 0.0 || !elapsed_ms.is_finite() {
    return AnnotationReveal::WHOLE;
  }
  let presence = |at: f32| counter_presence(at, duration_ms, opening_ms, closing_ms);
  let now = presence(elapsed_ms);
  let before = presence(elapsed_ms - frame_ms.max(0.0));
  AnnotationReveal {
    low: 0.0,
    high: 1.0,
    scale: now,
    opacity: now,
    previous: [0.0, 1.0, before, before],
  }
}
