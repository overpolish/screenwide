// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a moving annotation offers the snap engine, and the rectangles it can
//! align to.
//!
//! Aligning centres is a point's rule, and a counter is not a point: it is a
//! disc, and the rectangle and highlight tools draw boxes. A box aligns by
//! whichever of its two edges or its centre is nearest a candidate, so the
//! moving shape offers a small set of lines per axis and the engine picks one
//! offset for all of them. Letting each line take its own nearest guide would
//! tear the shape apart.

use super::AnnotationPoint;

/// Which axis a line or an extent belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
  X,
  Y,
}

impl Axis {
  /// The axis a gap's bars run across.
  pub(crate) fn other(self) -> Self {
    match self {
      Self::X => Self::Y,
      Self::Y => Self::X,
    }
  }
}

/// A rectangle in source pixels: something a gesture can align to, or the
/// moving shape itself.
///
/// Only annotations make field boxes. A detected UI element is a candidate
/// for a point subject - an arrow's tip, a counter's tail tip - because a
/// frame holds thousands of them, and offering their edges as alignment lines
/// would cost a pair of loops over the whole detection per sample and align a
/// disc to whatever noise the detector found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SnapBox {
  pub(crate) x: f64,
  pub(crate) y: f64,
  pub(crate) width: f64,
  pub(crate) height: f64,
}

impl SnapBox {
  /// A counter's disc, which is the whole of what a counter aligns by: its
  /// tail is a point subject of its own rather than part of the box.
  pub(crate) fn disc(center: AnnotationPoint, radius: f64) -> Self {
    Self {
      x: center.x - radius,
      y: center.y - radius,
      width: radius * 2.0,
      height: radius * 2.0,
    }
  }

  /// The near edge, the centre and the far edge, in that order.
  pub(crate) fn lines(&self, axis: Axis) -> [f64; 3] {
    [self.min(axis), self.centre(axis), self.max(axis)]
  }

  pub(crate) fn min(&self, axis: Axis) -> f64 {
    match axis {
      Axis::X => self.x,
      Axis::Y => self.y,
    }
  }

  pub(crate) fn max(&self, axis: Axis) -> f64 {
    self.min(axis) + self.extent(axis)
  }

  pub(crate) fn centre(&self, axis: Axis) -> f64 {
    self.min(axis) + self.extent(axis) / 2.0
  }

  fn extent(&self, axis: Axis) -> f64 {
    match axis {
      Axis::X => self.width,
      Axis::Y => self.height,
    }
  }

  /// The same rectangle carried by a snapped move.
  pub(crate) fn moved(&self, offset: SnapOffset) -> Self {
    Self {
      x: self.x + offset.x,
      y: self.y + offset.y,
      ..*self
    }
  }
}

/// The lines one axis of a moving shape offers. At most three - two edges and
/// a centre - and held inline, because a subject is built for every pointer
/// sample and none of them is worth an allocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SubjectLines {
  values: [f64; 3],
  len: usize,
}

impl SubjectLines {
  pub(crate) fn as_slice(&self) -> &[f64] {
    &self.values[..self.len]
  }
}

impl From<[f64; 3]> for SubjectLines {
  fn from(values: [f64; 3]) -> Self {
    Self { values, len: 3 }
  }
}

/// What the moving shape offers the engine, per axis. A box carried whole
/// offers all three of its lines; dragging one edge would offer only that
/// edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SnapSubject {
  pub(crate) x: SubjectLines,
  pub(crate) y: SubjectLines,
}

impl SnapSubject {
  /// A rectangle carried whole.
  pub(crate) fn moved_box(bounds: SnapBox) -> Self {
    Self {
      x: bounds.lines(Axis::X).into(),
      y: bounds.lines(Axis::Y).into(),
    }
  }
}

/// How far a snapped subject travels, in source pixels. One offset per axis
/// carries the whole shape, so every line of it keeps its place in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SnapOffset {
  pub(crate) x: f64,
  pub(crate) y: f64,
}

impl SnapOffset {
  pub(crate) fn apply(&self, point: AnnotationPoint) -> AnnotationPoint {
    AnnotationPoint {
      x: point.x + self.x,
      y: point.y + self.y,
    }
  }
}
