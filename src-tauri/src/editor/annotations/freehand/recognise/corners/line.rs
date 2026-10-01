// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A straight leg's line, fitted through the points the hand drew along it.

use crate::editor::annotations::AnnotationPoint;

/// How far on average a straight leg's points lie from the line fitted
/// through them, as a share of its length, at most. A hand's straight leg
/// lies within one percent; a piece of a curve strays by three or more.
const LEG_SHARE: f64 = 0.02;
/// The least angle, in degrees, two legs' lines meet at for the corner to be
/// where they cross.
const CROSSING: f64 = 20.0;

/// A leg's line: a point on it, its direction, and how long the leg runs.
pub(super) struct Line {
  pub(super) through: AnnotationPoint,
  pub(super) direction: [f64; 2],
  pub(super) length: f64,
}

impl Line {
  /// Where this line and `other` cross, or `None` where they meet at less
  /// than [`CROSSING`]: two such lines are all but one, and where they cross
  /// is a matter of the hand's jitter.
  pub(super) fn cross(&self, other: &Line) -> Option<AnnotationPoint> {
    let [ax, ay] = self.direction;
    let [bx, by] = other.direction;
    let determinant = ax * by - ay * bx;
    if determinant.abs() < CROSSING.to_radians().sin() {
      return None;
    }
    let (dx, dy) = (
      other.through.x - self.through.x,
      other.through.y - self.through.y,
    );
    let t = (dx * by - dy * bx) / determinant;
    Some(AnnotationPoint {
      x: self.through.x + ax * t,
      y: self.through.y + ay * t,
    })
  }

  /// The point on this line nearest `point`.
  pub(super) fn foot(&self, point: AnnotationPoint) -> AnnotationPoint {
    let [ax, ay] = self.direction;
    let t = (point.x - self.through.x) * ax + (point.y - self.through.y) * ay;
    AnnotationPoint {
      x: self.through.x + ax * t,
      y: self.through.y + ay * t,
    }
  }
}

/// The line through a leg's points, fitted across all of them, where they lie
/// along it as a straight leg does; `None` where they bow or are too few.
pub(super) fn straight_line(points: &[AnnotationPoint]) -> Option<Line> {
  if points.len() < 2 {
    return None;
  }
  let count = points.len() as f64;
  let mean = points.iter().fold([0.0, 0.0], |sum, point| {
    [sum[0] + point.x / count, sum[1] + point.y / count]
  });
  let (mut xx, mut xy, mut yy) = (0.0, 0.0, 0.0);
  for point in points {
    let (dx, dy) = (point.x - mean[0], point.y - mean[1]);
    xx += dx * dx;
    xy += dx * dy;
    yy += dy * dy;
  }
  // The direction the points spread along most, and how far they stray
  // across it on average.
  let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
  let [ax, ay] = [angle.cos(), angle.sin()];
  let (mut low, mut high, mut across) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
  for point in points {
    let (dx, dy) = (point.x - mean[0], point.y - mean[1]);
    let at = dx * ax + dy * ay;
    low = low.min(at);
    high = high.max(at);
    across += (dx * ay - dy * ax).powi(2) / count;
  }
  let length = high - low;
  (length > 0.0 && across.sqrt() <= LEG_SHARE * length).then_some(Line {
    through: AnnotationPoint {
      x: mean[0],
      y: mean[1],
    },
    direction: [ax, ay],
    length,
  })
}
