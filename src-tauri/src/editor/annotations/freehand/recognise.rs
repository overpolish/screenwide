// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a stroke held still at its end is taken for.
//!
//! A hand that rests at the end of a stroke is asking for the clean version of
//! what it drew. A closed stroke drawn with straight sides is those lines, or
//! a box shape where it is an upright four-sided one; one that follows an
//! ellipse is that ellipse. An open one is read as a line with a hook at its
//! end, which is an arrow with its head; then as straight legs between its
//! turns; then as a line or an even curve, which is an arrow without one.
//! Anything else is only smoothed.
//!
//! Every threshold is a share of the stroke's own size with a floor in screen
//! points, so a large stroke is read by its shape and a small one is not read
//! from the jitter in it. `unit` is source pixels per screen point.

use crate::editor::annotations::AnnotationPoint;

/// Reading an open stroke as an arrow with its head drawn on.
mod arrow;
use arrow::hooked_arrow;
/// Reading a closed stroke.
mod closed;
/// Where a stroke turns, and the straight lines it runs between.
mod corners;
/// The ellipse a loop encloses.
pub(crate) mod ellipse;
use closed::{closing, outline};

/// What a held stroke becomes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Recognised {
  /// An arrow from `start` to `end`, bent by the Bézier `control`, with a
  /// head at its end where `head`.
  Arrow {
    start: AnnotationPoint,
    control: AnnotationPoint,
    end: AnnotationPoint,
    head: bool,
  },
  /// A square-cornered box from `low` to `high`.
  Box {
    low: AnnotationPoint,
    high: AnnotationPoint,
  },
  /// A shape from `low` to `high` rounded fully: a circle over a square box,
  /// a pill over any other.
  Round {
    low: AnnotationPoint,
    high: AnnotationPoint,
  },
  /// An ellipse, turned as the loop was drawn, where it leans too far for
  /// a pill to stand in for it.
  Ellipse(ellipse::Ellipse),
  /// Straight lines through these points in turn, where the hand turned
  /// between legs it drew straight. A closed one ends where it began.
  Lines { points: Vec<AnnotationPoint> },
  /// A smooth curve through these points in turn: the stroke's own shape,
  /// with the jerks a mouse or a trackpad put in it taken out. A closed one
  /// ends where it began.
  Curve { points: Vec<AnnotationPoint> },
}

/// The shortest stroke, in screen points, read as anything: below it a rest
/// is a dot, and a dot is left as drawn.
const SMALLEST: f64 = 16.0;
/// How far a line may stray from straight, or from an even curve, in screen
/// points however short it is.
const STRAY: f64 = 4.0;
/// How far a line may stray from straight, or from an even curve, as a share
/// of the distance between its ends.
const STRAY_SHARE: f64 = 0.07;
/// How much longer than the line it is read as a stroke may run: a hand that
/// went back and forth along one line drew a scribble, not a line.
const LENGTH_SHARE: f64 = 1.3;
/// How near a stroke must come back to its start, as a share of the longer
/// side of the box round it, for it to be closed. A loop left open by more
/// than a sliver is a curve, however round.
const CLOSE_SHARE: f64 = 0.15;
/// How near a stroke must come back to its start, as the same share, to be
/// a box left open at a corner: a hand often stops short of the corner it
/// began at, where a round loop left that open is a curve.
const OPEN_BOX_SHARE: f64 = 0.3;

/// What `points`, held still at their end, are taken for, or `None` for a
/// stroke too small to read.
pub(crate) fn recognise(points: &[AnnotationPoint], unit: f64) -> Option<Recognised> {
  let unit = if unit.is_finite() && unit > 0.0 {
    unit
  } else {
    1.0
  };
  let length = path_length(points);
  if points.len() < 2 || length < SMALLEST * unit {
    return None;
  }
  // Closed wherever the stroke comes back round near its start, so a hand
  // that overshoots it is closed as surely as one that stops short.
  let (low, high) = super::model::bounds(points);
  let size = (high.x - low.x).max(high.y - low.y);
  let close = closing(points);
  if distance(points[0], points[close]) <= CLOSE_SHARE * size {
    return Some(outline(points, unit).unwrap_or_else(|| closed_curve(&points[..=close])));
  }
  // An end heading straight for the line the stroke began with closes it
  // there, however far short of it the hand stopped.
  if let Some(ring) = closed::closed_by_reach(points, size) {
    if let Some(read) = outline(&ring, unit) {
      return Some(read);
    }
  }
  if distance(points[0], points[close]) <= OPEN_BOX_SHARE * size {
    if let Some(boxed @ Recognised::Box { .. }) = outline(points, unit) {
      return Some(boxed);
    }
  }
  if let Some(hooked) = hooked_arrow(points, unit) {
    return Some(hooked);
  }
  // One line or one even curve is an arrow; failing that, a hand that turned
  // between straight legs drew those lines, an L or a zigzag.
  if let Some((start, control, end)) = fitted(points, unit) {
    return Some(Recognised::Arrow {
      start,
      control,
      end,
      head: false,
    });
  }
  let walk = corners::walk(points, false);
  let turns = walk.turns();
  if turns.len() > 2 {
    if let Some(points) = walk.straightened(&turns) {
      return Some(Recognised::Lines { points });
    }
  }
  Some(Recognised::Curve {
    points: corners::walk(points, false).curve(),
  })
}

/// A smooth closed curve round the loop `ring`, closed back to its first
/// point.
fn closed_curve(ring: &[AnnotationPoint]) -> Recognised {
  let mut points = corners::walk(ring, true).curve();
  points.push(points[0]);
  Recognised::Curve { points }
}

/// `points` as one arrow's line from the first to the last: straight where
/// they stray little from the chord, otherwise the quadratic through their
/// middle, and `None` where they follow neither.
fn fitted(
  points: &[AnnotationPoint],
  unit: f64,
) -> Option<(AnnotationPoint, AnnotationPoint, AnnotationPoint)> {
  let (start, end) = (points[0], points[points.len() - 1]);
  let chord = distance(start, end);
  if chord < SMALLEST * unit {
    return None;
  }
  let length = path_length(points);
  let tolerance = (STRAY * unit).max(STRAY_SHARE * chord);
  let straight = points
    .iter()
    .all(|point| segment_distance(*point, start, end) <= tolerance);
  if straight && length <= LENGTH_SHARE * chord {
    return Some((start, midpoint(start, end), end));
  }
  // The quadratic that passes through the stroke's middle halfway along it,
  // which is where an arrow's middle grip sits.
  let middle = along(points, length / 2.0);
  let control = AnnotationPoint {
    x: 2.0 * middle.x - (start.x + end.x) / 2.0,
    y: 2.0 * middle.y - (start.y + end.y) / 2.0,
  };
  let curve: Vec<AnnotationPoint> = (0..=64)
    .map(|step| quadratic(start, control, end, f64::from(step) / 64.0))
    .collect();
  let follows = points.iter().all(|point| {
    curve
      .windows(2)
      .map(|pair| segment_distance(*point, pair[0], pair[1]))
      .fold(f64::INFINITY, f64::min)
      <= tolerance
  });
  (follows && length <= LENGTH_SHARE * path_length(&curve)).then_some((start, control, end))
}

fn distance(a: AnnotationPoint, b: AnnotationPoint) -> f64 {
  (b.x - a.x).hypot(b.y - a.y)
}

fn midpoint(a: AnnotationPoint, b: AnnotationPoint) -> AnnotationPoint {
  AnnotationPoint {
    x: (a.x + b.x) / 2.0,
    y: (a.y + b.y) / 2.0,
  }
}

fn path_length(points: &[AnnotationPoint]) -> f64 {
  points
    .windows(2)
    .map(|pair| distance(pair[0], pair[1]))
    .sum()
}

/// The point `at` along the stroke's path from its first point.
fn along(points: &[AnnotationPoint], at: f64) -> AnnotationPoint {
  let mut left = at;
  for pair in points.windows(2) {
    let leg = distance(pair[0], pair[1]);
    if leg > 0.0 && left <= leg {
      let share = left / leg;
      return AnnotationPoint {
        x: pair[0].x + (pair[1].x - pair[0].x) * share,
        y: pair[0].y + (pair[1].y - pair[0].y) * share,
      };
    }
    left -= leg;
  }
  points[points.len() - 1]
}

fn segment_distance(point: AnnotationPoint, a: AnnotationPoint, b: AnnotationPoint) -> f64 {
  let (dx, dy) = (b.x - a.x, b.y - a.y);
  let squared = dx * dx + dy * dy;
  let share = if squared > 0.0 {
    (((point.x - a.x) * dx + (point.y - a.y) * dy) / squared).clamp(0.0, 1.0)
  } else {
    0.0
  };
  distance(
    point,
    AnnotationPoint {
      x: a.x + dx * share,
      y: a.y + dy * share,
    },
  )
}

fn quadratic(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  t: f64,
) -> AnnotationPoint {
  let u = 1.0 - t;
  AnnotationPoint {
    x: u * u * start.x + 2.0 * u * t * control.x + t * t * end.x,
    y: u * u * start.y + 2.0 * u * t * control.y + t * t * end.y,
  }
}
