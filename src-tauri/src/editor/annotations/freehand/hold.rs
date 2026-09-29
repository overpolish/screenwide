// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Holding a fresh stroke still at its end, to have it taken for the clean
//! annotation it looks like.
//!
//! The pointer reports nothing while it rests, so the rest is timed from the
//! last sample that moved and read by a clock outside the gesture, which asks
//! [`StrokeHold::hold`] until the stroke ends. What the stroke is taken for
//! replaces it while the button is still down, so the hand sees it before
//! letting go; moving on gives the stroke back, and it carries on as drawn.

use std::time::{Duration, Instant};

use super::recognise::{recognise, Recognised};
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationPoint, AnnotationShape};

/// How long a hand rests at the end of a stroke to have it read.
pub(crate) const HOLD: Duration = Duration::from_millis(500);
/// How far a resting hand may wander, in screen points, and still be resting.
const REST: f64 = 4.0;
/// How many points an ellipse is drawn through: enough that the smoothed
/// stroke through them is the ellipse to the eye.
const ELLIPSE_POINTS: usize = 48;
/// A fully rounded shape's radius, in percent of its shorter side: a square
/// one is a circle.
const ROUND: f64 = 50.0;

/// One fresh stroke's rest: where the hand came to rest and since when, and
/// the stroke as drawn while what it was taken for stands in its place.
pub(crate) struct StrokeHold {
  rest: AnnotationPoint,
  since: Instant,
  /// This rest has been read, whether or not anything came of it.
  read: bool,
  drawn: Option<Annotation>,
}

impl StrokeHold {
  pub(crate) fn new(point: AnnotationPoint, now: Instant) -> Self {
    Self {
      rest: point,
      since: now,
      read: false,
      drawn: None,
    }
  }

  /// One pointer sample, `unit` source pixels to the screen point. Moving on
  /// from the rest restarts its clock and gives the stroke as drawn back.
  /// Answers whether the sample carries the stroke on: one that stays at a
  /// rest that has been read leaves what it was taken for in place.
  pub(crate) fn sample(
    &mut self,
    annotation: &mut Annotation,
    point: AnnotationPoint,
    unit: f64,
    now: Instant,
  ) -> bool {
    let reach = REST * if unit > 0.0 { unit } else { 1.0 };
    if (point.x - self.rest.x).hypot(point.y - self.rest.y) <= reach {
      return self.drawn.is_none();
    }
    self.rest = point;
    self.since = now;
    self.read = false;
    if let Some(drawn) = self.drawn.take() {
      *annotation = drawn;
    }
    true
  }

  /// Reads the stroke once the hand has rested on it for [`HOLD`], and puts
  /// what it was taken for in its place. Answers whether the annotation
  /// changed.
  pub(crate) fn hold(&mut self, annotation: &mut Annotation, unit: f64, now: Instant) -> bool {
    if self.read || now.saturating_duration_since(self.since) < HOLD {
      return false;
    }
    self.read = true;
    let AnnotationShape::Draw { points, .. } = &annotation.shape else {
      return false;
    };
    let Some(taken) = recognise(points, unit) else {
      return false;
    };
    let drawn = annotation.clone();
    *annotation = taken_for(&drawn, taken);
    self.drawn = Some(drawn);
    true
  }
}

/// The annotation `taken` stands for, in the stroke's own colour and pen, so
/// a stroke read as an arrow or an outline looks the weight it was drawn.
fn taken_for(drawn: &Annotation, taken: Recognised) -> Annotation {
  let mut annotation = drawn.clone();
  annotation.style.hand_drawn = false;
  annotation.style.manual = false;
  match taken {
    Recognised::Arrow {
      start,
      control,
      end,
      head,
    } => {
      annotation.style.head = if head {
        AnnotationHead::End
      } else {
        AnnotationHead::None
      };
      annotation.shape = AnnotationShape::Arrow {
        start,
        control,
        end,
      };
      crate::editor::annotations::arrow::bend::clamp_bend(&mut annotation);
    }
    Recognised::Box { low, high } => as_shape(&mut annotation, low, high, 0.0),
    // A box rounded fully is a circle or a pill, and the shape tool's own.
    Recognised::Round { low, high } => as_shape(&mut annotation, low, high, ROUND),
    // Still a stroke, drawn through its corners alone: the fitted line keeps
    // a drawn stroke's corners all but sharp, and it edits as any stroke does.
    Recognised::Lines { points } => {
      annotation.shape = AnnotationShape::Draw {
        points,
        smooth: false,
      };
    }
    // A shape rounds a box, which draws an upright pill rather than an
    // oval, so an oval is a stroke smoothed through points round it.
    Recognised::Ellipse(ellipse) => {
      annotation.shape = AnnotationShape::Draw {
        points: ellipse.outline(ELLIPSE_POINTS),
        smooth: true,
      };
    }
    Recognised::Curve { points } => {
      annotation.shape = AnnotationShape::Draw {
        points,
        smooth: true,
      };
    }
  }
  annotation
}

/// Makes `annotation` a shape from `low` to `high`, its corners rounded by
/// `radius` percent of its shorter side.
fn as_shape(annotation: &mut Annotation, low: AnnotationPoint, high: AnnotationPoint, radius: f64) {
  annotation.style.head = AnnotationHead::None;
  annotation.style.radius = radius;
  annotation.shape = AnnotationShape::Shape {
    start: low,
    end: high,
    seed: crate::editor::annotations::highlight::model::fresh_seed(),
  };
}
