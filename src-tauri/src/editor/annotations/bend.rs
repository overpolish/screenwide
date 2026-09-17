// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The arrow's curve, and how far it may be bent.
//!
//! An arrow is one quadratic Bézier, and its bend is one number in the
//! chord's own terms: how far off the chord the curve's midpoint sits, as a
//! fraction of the chord's length. The midpoint is always halfway along the
//! chord - the curve's control point rides the perpendicular bisector and
//! nothing else.
//!
//! That is the whole shape of an arrow worth drawing. Sliding the midpoint
//! along the chord instead only shortens one half of the curve against the
//! other, which reads as a pointless kink rather than a bend, and as the
//! control point nears a tip the tangent there turns to noise and swings the
//! head off the shaft. Held as one number the bend also belongs to the arrow
//! rather than to the canvas, so dragging a tip carries the curve round with
//! the shaft, and the one limit that keeps a curve drawable - a hairpin short
//! of a loop - is a number to clamp rather than a shape to test.

use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// How far off the chord the curve's midpoint may sit, as a fraction of the
/// chord: far enough for a hairpin, not far enough to close into a loop.
const BEND_ACROSS_MAXIMUM: f64 = 0.6;
/// A chord shorter than one source pixel names no direction to bend across.
const MINIMUM_CHORD: f64 = 1.0;

/// The point a quadratic Bézier passes through at `t = 0.5`.
pub(crate) fn curve_midpoint(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
) -> AnnotationPoint {
  AnnotationPoint {
    x: 0.25 * start.x + 0.5 * control.x + 0.25 * end.x,
    y: 0.25 * start.y + 0.5 * control.y + 0.25 * end.y,
  }
}

/// The control point that makes the curve pass through `middle` at `t = 0.5`:
/// the inverse of [`curve_midpoint`].
pub(crate) fn control_through_midpoint(
  start: AnnotationPoint,
  middle: AnnotationPoint,
  end: AnnotationPoint,
) -> AnnotationPoint {
  AnnotationPoint {
    x: 2.0 * middle.x - (start.x + end.x) / 2.0,
    y: 2.0 * middle.y - (start.y + end.y) / 2.0,
  }
}

/// How far off its own chord an arrow's curve swings, as a signed fraction of
/// the chord's length. The sign is which side of the chord it swings to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ArrowBend {
  pub(crate) across: f64,
}

impl ArrowBend {
  /// A straight arrow: the midpoint on the chord.
  pub(crate) const STRAIGHT: Self = Self { across: 0.0 };

  /// The bend as far as an arrow may be bent.
  pub(crate) fn clamped(self) -> Self {
    Self {
      across: if self.across.is_finite() {
        self.across.clamp(-BEND_ACROSS_MAXIMUM, BEND_ACROSS_MAXIMUM)
      } else {
        0.0
      },
    }
  }
}

/// The chord an arrow is bent across: how long it is, and the unit direction
/// across it, which is the only one the bend is measured in.
struct ChordFrame {
  length: f64,
  across: (f64, f64),
}

/// The frame of the chord from `start` to `end`. `None` for a chord too short
/// to point anywhere.
fn chord_frame(start: AnnotationPoint, end: AnnotationPoint) -> Option<ChordFrame> {
  let (dx, dy) = (end.x - start.x, end.y - start.y);
  let length = dx.hypot(dy);
  if !length.is_finite() || length < MINIMUM_CHORD {
    return None;
  }
  let along = (dx / length, dy / length);
  Some(ChordFrame {
    length,
    across: (-along.1, along.0),
  })
}

/// What the arrow's bend measures, exactly as it is drawn: the curve's
/// midpoint projected onto the chord's perpendicular, which is all of the
/// midpoint the bend keeps. Unclamped - this is the reading, and
/// [`ArrowBend::clamped`] is the rule applied to it.
pub(crate) fn arrow_bend(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
) -> ArrowBend {
  let Some(chord) = chord_frame(start, end) else {
    return ArrowBend::STRAIGHT;
  };
  let middle = curve_midpoint(start, control, end);
  let (dx, dy) = (middle.x - start.x, middle.y - start.y);
  ArrowBend {
    across: (dx * chord.across.0 + dy * chord.across.1) / chord.length,
  }
}

/// The control point that bends this chord by `bend`: the curve's midpoint
/// taken off the middle of the chord, along its perpendicular, and solved
/// back. A chord with no direction to bend across is drawn straight.
pub(crate) fn control_for_bend(
  start: AnnotationPoint,
  end: AnnotationPoint,
  bend: ArrowBend,
) -> AnnotationPoint {
  let centre = AnnotationPoint {
    x: (start.x + end.x) / 2.0,
    y: (start.y + end.y) / 2.0,
  };
  let Some(chord) = chord_frame(start, end) else {
    return centre;
  };
  let middle = AnnotationPoint {
    x: centre.x + chord.length * bend.across * chord.across.0,
    y: centre.y + chord.length * bend.across * chord.across.1,
  };
  control_through_midpoint(start, middle, end)
}

/// Puts an arrow's curve back on the chord's perpendicular bisector, and back
/// within the hairpin. A counter has no curve to repair.
///
/// Applied to every edit, so a document written before the limits existed is
/// repaired the first time its arrow is touched rather than at load: nothing
/// silently rewrites a file that is only being looked at.
pub(crate) fn clamp_bend(annotation: &mut Annotation) {
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = &mut annotation.shape
  else {
    return;
  };
  *control = control_for_bend(*start, *end, arrow_bend(*start, *control, *end).clamped());
}
