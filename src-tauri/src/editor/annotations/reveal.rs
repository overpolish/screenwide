// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reveal a timed annotation draws itself in and out through.
//!
//! An annotation is drawn the way a hand draws one: the stroke leaves the tail
//! and runs to the head, which rides the end that is moving until it lands on
//! the end point. The close carries on in the same direction - the tail catches
//! up to the head, and the annotation leaves from where it was pointing.
//!
//! Two things carry that, and each takes its own part of the phase. The
//! window - `low` to `high` - is how much of the path the annotation covers,
//! in *arc length* rather than in the curve's parameter, because a quadratic
//! Bézier's parameter runs unevenly along a bend: revealing by parameter would
//! crawl through the curve's tight side and race through its slack side.
//! `scale` is the annotation's own size, and it exists because an annotation
//! can never be drawn shorter than its own head: travel alone holds a
//! full-sized head through the whole phase and then crosses that last
//! head-length in the two frames an eased phase spends near rest, which reads
//! as the head popping on and off rather than arriving and leaving.
//!
//! So an annotation grows to full weight as it sets off - stroke, heads and
//! their rounding together, a small whole arrow rather than the full-width
//! round cap a shaft of no length draws - and shrinks away as it finishes
//! leaving.
//!
//! Everything here is derived from the clip's bounds and the frame's source
//! time, so scrubbing backwards lands on exactly the frame playing forwards
//! drew. Nothing is integrated across frames.
//!
//! The compositor calls [`screenwide_annotation_reveal_geometry`] while it
//! prepares an annotation, in the canvas pixel space where the stroke's width
//! and the curve's length are finally comparable, and the video export calls
//! [`screenwide_annotation_reveal_window`] once per frame per clip. Preview and
//! export therefore animate through this one implementation.

#[cfg(target_os = "macos")]
use crate::editor::annotations::AnnotationKind;
use crate::editor::effect_animation::ease_in_out_cubic;

/// How long an annotation takes to draw itself in. The twin of
/// `ANNOTATION_DRAW_IN_MS` in `src/features/editor/annotations.ts`, which
/// places a fresh clip this far before the playhead so the annotation is drawn
/// by the time the playhead is reached;
/// `the_editor_places_a_clip_by_this_phase` holds the two together. A second:
/// the drawing is the thing a viewer is meant to follow to what the arrow
/// points at, and it is gentler for being given its time.
pub(crate) const REVEAL_DRAW_IN_MS: f32 = 1_000.0;

/// How long an annotation takes to undraw. Shorter than the drawing: leaving is
/// not the thing being watched, and an annotation that lingers on its way out
/// is in the way of whatever comes next.
pub(crate) const REVEAL_DRAW_OUT_MS: f32 = 750.0;

/// The most of a clip either phase may take, so a clip shorter than a second
/// still finishes drawing itself in before it starts leaving.
const REVEAL_PHASE_SHARE: f32 = 1.0 / 3.0;

/// The share of the opening phase the annotation fades in over, from the clip's
/// start, while it is already under way. It arrives whole: a head that grows in
/// is a head two hundred pixels long changing shape over a handful of frames on
/// a heavy annotation, and a stroke that grows in weight is one that looks
/// drawn twice. Fading, nothing changes shape - the arrow is simply there,
/// already drawing, as the eye finds it. Long enough that the stroke is a width
/// or more long before the annotation is half solid: shorter than that its
/// round end is a nub on the back of the head, however it is held in.
const REVEAL_FADE_SHARE: f32 = 0.3;

/// The share of the closing phase the annotation shrinks away over, at the
/// clip's end. Longer than the arrival: the head starts to go while the stroke
/// is still drawing itself back in, so the two leave as one motion rather than
/// the stroke finishing and the head following.
const REVEAL_SHRINK_SHARE: f32 = 0.35;

/// How far into its ease the opening travel sets off from. An in-out ease
/// spends its first stretch all but standing still, so a head that grows in
/// while the stroke is that far in has finished growing before anything has
/// moved, which reads as an arrow inflated and then pushed. Setting off a
/// little way in, the stroke is already moving on the first frame - at about
/// a sixth of its average speed - and still gathers to its peak. Further in
/// and the stroke has drawn a long way before the head is whole, which reads
/// as a stem with a head on the end rather than an arrow setting off.
const REVEAL_TAKEOFF: f32 = 0.12;

/// The share of the closing phase the stroke takes to draw itself back into
/// the head; the rest is the head alone. Landing before the phase ends is
/// what keeps the stroke moving while the head shrinks: an ease that has to
/// come to rest at the very end spends the head's whole shrink creeping over
/// its last few pixels, which reads as a stroke that has stopped.
const REVEAL_LAND_SHARE: f32 = 0.9;

/// Current reveal and shutter-start state, in arc-length fractions.
/// The previous state carries low, high, scale and opacity in that order.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnnotationReveal {
  pub low: f32,
  pub high: f32,
  pub scale: f32,
  pub opacity: f32,
  pub previous: [f32; 4],
}

impl AnnotationReveal {
  /// The whole path at full size, standing still: what a screenshot, an
  /// annotation that is not animated, and the middle of every animated clip all
  /// draw.
  pub const WHOLE: Self = Self {
    low: 0.0,
    high: 1.0,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, 1.0, 1.0, 1.0],
  };

  /// Whether this is the whole path standing still, which the compositor
  /// prepares through its original static path rather than the reveal's.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub fn is_whole(self) -> bool {
    self.low == 0.0 && self.high == 1.0 && self.scale == 1.0 && self.opacity == 1.0
  }
}

impl Default for AnnotationReveal {
  fn default() -> Self {
    Self::WHOLE
  }
}

/// The reveal `elapsed_ms` into a clip lasting `duration_ms`.
///
/// `frame_ms` is the exposure interval in source time. A paused preview uses
/// zero.
pub(crate) fn reveal_window(elapsed_ms: f32, duration_ms: f32, frame_ms: f32) -> AnnotationReveal {
  let opening_ms = REVEAL_DRAW_IN_MS.min(duration_ms * REVEAL_PHASE_SHARE);
  let closing_ms = REVEAL_DRAW_OUT_MS.min(duration_ms * REVEAL_PHASE_SHARE);
  if !opening_ms.is_finite() || opening_ms <= 0.0 || !elapsed_ms.is_finite() {
    return AnnotationReveal::WHOLE;
  }
  // Both ends ease in *and* out, rather than easing out the way the ruler's
  // hover halo and the keyboard's pops do. A pulse announces itself: it is over
  // before it is read, so it has to start at its fastest. An annotation drawing
  // itself is read while it moves, and an ease-out leaves the stroke at three
  // times its average speed on the very first frame, which reads as the arrow
  // being launched rather than drawn. In-out starts and stops gently and spends
  // its speed in the middle, where the eye is already following.
  //
  // The phases never overlap - each is at most a third of the clip - so the end
  // that is not running sits at rest on its own extreme.
  //
  // The annotation arrives whole and fades in while it is already drawing;
  // leaving, its head starts to go while the stroke is still drawing itself
  // back in - and still moving, the stroke being home before the head has half
  // gone.
  let fade_ms = opening_ms * REVEAL_FADE_SHARE;
  let shrink_ms = closing_ms * REVEAL_SHRINK_SHARE;
  let land_ms = closing_ms * REVEAL_LAND_SHARE;
  let takeoff = ease_in_out_cubic(REVEAL_TAKEOFF);
  let ends = |at_ms: f32| {
    let opening = REVEAL_TAKEOFF + (1.0 - REVEAL_TAKEOFF) * (at_ms / opening_ms).clamp(0.0, 1.0);
    (
      ease_in_out_cubic(((at_ms - (duration_ms - closing_ms)) / land_ms).clamp(0.0, 1.0)),
      (ease_in_out_cubic(opening) - takeoff) / (1.0 - takeoff),
    )
  };
  // One formula for the window, weight and opacity at a source time, so the
  // current frame and the one just before it - the blur's lead - share it
  // rather than the previous state repeating the easing inline.
  let state = |at_ms: f32| {
    let (low, high) = ends(at_ms);
    let arriving = ease_in_out_cubic((at_ms / fade_ms).clamp(0.0, 1.0));
    let leaving =
      ease_in_out_cubic(((at_ms - (duration_ms - shrink_ms)) / shrink_ms).clamp(0.0, 1.0));
    (low, high, 1.0 - leaving, arriving)
  };
  let (low, high, scale, opacity) = state(elapsed_ms);
  let (previous_low, previous_high, previous_scale, previous_opacity) =
    state(elapsed_ms - frame_ms.max(0.0));
  AnnotationReveal {
    low,
    high,
    scale,
    opacity,
    previous: [
      previous_low,
      previous_high,
      previous_scale,
      previous_opacity,
    ],
  }
}

/// The clip length that leaves an animated annotation starting to draw out
/// exactly `visible_ms` after it appeared.
///
/// A live annotation is on screen from its stroke until it is cleared, and
/// then it is simply gone. A clip that ends at the clear runs its whole
/// closing phase *before* that moment, so the annotation would be leaving
/// while it was still whole on screen. Carrying the closing phase's own
/// length on top puts the leaving where it belongs: after the annotation
/// actually went. A short clip needs less than the full
/// [`REVEAL_DRAW_OUT_MS`], because its closing is capped to a third of it.
pub(crate) fn clip_ms_for_visible(visible_ms: f32) -> f32 {
  if !visible_ms.is_finite() || visible_ms <= 0.0 {
    return 0.0;
  }
  let uncapped = visible_ms + REVEAL_DRAW_OUT_MS;
  if uncapped * REVEAL_PHASE_SHARE >= REVEAL_DRAW_OUT_MS {
    uncapped
  } else {
    visible_ms / (1.0 - REVEAL_PHASE_SHARE)
  }
}

/// The reveal window one clip is at, for the video export's per-frame pass. An
/// annotation that is not animated is drawn whole for the clip's whole length,
/// and every other one follows its own kind's arrival: `kind` is the retained
/// annotation's, so the export and the preview animate through one
/// implementation.
///
/// # Safety
/// `out` must point at one writable [`AnnotationReveal`].
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_annotation_reveal_window(
  elapsed_ms: f32,
  duration_ms: f32,
  frame_ms: f32,
  animated: u32,
  kind: u32,
  out: *mut AnnotationReveal,
) {
  if out.is_null() {
    return;
  }
  *out = if animated == 0 {
    AnnotationReveal::WHOLE
  } else {
    // A number no kind owns cannot come from a retained annotation, and the
    // arrow's window is what such a record drew before the kinds were named.
    let kind = AnnotationKind::from_raw(kind).unwrap_or(AnnotationKind::Arrow);
    kind.reveal_window(elapsed_ms, duration_ms, frame_ms)
  };
}

/// Turning a window into drawable geometry needs the curve itself, and only
/// the compositor has one.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[path = "reveal_geometry.rs"]
pub(crate) mod geometry;

#[cfg(test)]
#[path = "reveal_tests.rs"]
mod tests;
