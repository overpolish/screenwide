// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How long an annotation's path takes to draw in on a recording, from how
//! long the path is: the pace a clip keeps as `path_ms`.
//!
//! The editor paces every clip it writes, in
//! `src/features/editor/annotation-pace.ts`, which explains the choice of
//! curve. This is its twin for the clips made here rather than there - the
//! live overlay's - so an annotation drawn live arrives at the pace the same
//! annotation drawn in the editor would. Both hold the same cases in their
//! tests.

use super::reveal::REVEAL_DRAW_IN_MS;
use super::text::reveal::POINTER_IN_MS;
use super::{Annotation, AnnotationPoint, AnnotationShape};

/// The share of the picture's diagonal a path drawn in the unpaced second
/// runs: about a typical arrow's.
const REFERENCE_SHARE: f64 = 0.2;
const PATH_MIN_MS: f64 = 600.0;
const PATH_MAX_MS: f64 = 2_000.0;

/// The reach in ems a pointer drawn in its unpaced time runs, and the least
/// and most any pointer takes.
const POINTER_REFERENCE_EMS: f64 = 3.0;
const POINTER_MIN_MS: f64 = 200.0;
const POINTER_MAX_MS: f64 = 900.0;

/// `base` grown by the square root of `ratio`, held to `min..=max`, in whole
/// milliseconds.
fn paced(base: f32, ratio: f64, min: f64, max: f64) -> f32 {
  (f64::from(base) * ratio.sqrt()).clamp(min, max).round() as f32
}

/// How far along a quadratic Bézier from `a` to `c` bent by `b`, over the
/// same thirty-two chords the editor measures.
fn curve_length(a: AnnotationPoint, b: AnnotationPoint, c: AnnotationPoint) -> f64 {
  let at = |t: f64| {
    let u = 1.0 - t;
    (
      u * u * a.x + 2.0 * u * t * b.x + t * t * c.x,
      u * u * a.y + 2.0 * u * t * b.y + t * t * c.y,
    )
  };
  let mut previous = (a.x, a.y);
  (1..=32)
    .map(|step| {
      let next = at(f64::from(step) / 32.0);
      let chord = (next.0 - previous.0).hypot(next.1 - previous.1);
      previous = next;
      chord
    })
    .sum()
}

/// How far an annotation's path runs, in source pixels, or `None` for one
/// that travels no path of its own there.
fn path_length(annotation: &Annotation) -> Option<f64> {
  match &annotation.shape {
    AnnotationShape::Arrow {
      start,
      control,
      end,
    } => Some(curve_length(*start, *control, *end)),
    AnnotationShape::Highlight { bands, .. } => Some(
      bands
        .iter()
        .map(|band| (band.right - band.left).max(0.0))
        .sum(),
    ),
    AnnotationShape::Shape { start, end, .. } => {
      let across = (end.x - start.x).abs();
      let down = (end.y - start.y).abs();
      let rounding = across.min(down) * annotation.style.radius.clamp(0.0, 50.0) / 100.0;
      Some(2.0 * (across + down) - 8.0 * rounding + std::f64::consts::TAU * rounding)
    }
    AnnotationShape::Counter { .. }
    | AnnotationShape::Redact { .. }
    | AnnotationShape::Spotlight { .. }
    | AnnotationShape::Text { .. } => None,
  }
}

/// How long `annotation`'s path takes to draw in, on a picture `frame` source
/// pixels in size, or `None` for one with no path, which arrives on its
/// kind's own time.
pub(crate) fn path_ms(annotation: &Annotation, frame: (u32, u32)) -> Option<f32> {
  if let AnnotationShape::Text { pointer, .. } = &annotation.shape {
    let reach = pointer.reach.x.hypot(pointer.reach.y);
    return (reach > 0.0).then(|| {
      paced(
        POINTER_IN_MS,
        reach / POINTER_REFERENCE_EMS,
        POINTER_MIN_MS,
        POINTER_MAX_MS,
      )
    });
  }
  let length = path_length(annotation)?;
  let sized = frame.0 > 0 || frame.1 > 0;
  (sized && length.is_finite()).then(|| travel_ms(length, frame))
}

/// How long something takes to travel `length` source pixels on a picture
/// `frame` source pixels in size, at the pace a path draws in: a spotlight's
/// light gliding to the next box keeps it too. A picture of unknown size
/// takes the unpaced time.
pub(crate) fn travel_ms(length: f64, frame: (u32, u32)) -> f32 {
  let diagonal = f64::from(frame.0).hypot(f64::from(frame.1));
  if diagonal <= 0.0 || !length.is_finite() {
    return REVEAL_DRAW_IN_MS;
  }
  paced(
    REVEAL_DRAW_IN_MS,
    length / diagonal / REFERENCE_SHARE,
    PATH_MIN_MS,
    PATH_MAX_MS,
  )
}

#[cfg(test)]
#[path = "pace_tests.rs"]
mod tests;
