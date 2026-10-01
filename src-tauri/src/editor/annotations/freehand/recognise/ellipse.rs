// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ellipse a closed stroke encloses the same area as, turned the way it
//! is turned.
//!
//! An ellipse's area has a quarter of its squared radii as its second
//! moments, so the region the loop encloses gives the ellipse directly: its
//! centre, how far it reaches along its two axes, and which way it leans.
//! The fit reads the whole region rather than the line round it, so a hand's
//! wobble, which adds as much area as it takes, hardly moves it.

use crate::editor::annotations::AnnotationPoint;

/// An ellipse: its centre, its two radii, and the angle, in radians, its
/// first radius lies along.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ellipse {
  pub(crate) centre: AnnotationPoint,
  pub(crate) radii: [f64; 2],
  pub(crate) angle: f64,
}

impl Ellipse {
  /// The ellipse whose area has the same moments as the loop `points`, or
  /// `None` for a loop that encloses nothing.
  pub(super) fn enclosed_by(points: &[AnnotationPoint]) -> Option<Self> {
    let count = points.len();
    let (mut area, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for index in 0..count {
      let (a, b) = (points[index], points[(index + 1) % count]);
      let cross = a.x * b.y - b.x * a.y;
      area += cross;
      cx += (a.x + b.x) * cross;
      cy += (a.y + b.y) * cross;
    }
    area /= 2.0;
    if area.abs() < f64::EPSILON {
      return None;
    }
    let centre = AnnotationPoint {
      x: cx / (6.0 * area),
      y: cy / (6.0 * area),
    };
    let (mut xx, mut yy, mut xy) = (0.0, 0.0, 0.0);
    for index in 0..count {
      let (a, b) = (points[index], points[(index + 1) % count]);
      let (x0, y0, x1, y1) = (
        a.x - centre.x,
        a.y - centre.y,
        b.x - centre.x,
        b.y - centre.y,
      );
      let cross = x0 * y1 - x1 * y0;
      xx += (x0 * x0 + x0 * x1 + x1 * x1) * cross;
      yy += (y0 * y0 + y0 * y1 + y1 * y1) * cross;
      xy += (x0 * y1 + 2.0 * x0 * y0 + 2.0 * x1 * y1 + x1 * y0) * cross;
    }
    // The region's spread along each axis, and across them.
    let (xx, yy, xy) = (xx / (12.0 * area), yy / (12.0 * area), xy / (24.0 * area));
    let half = (xx + yy) / 2.0;
    let apart = (((xx - yy) / 2.0).powi(2) + xy * xy).sqrt();
    Some(Self {
      centre,
      radii: [
        2.0 * (half + apart).max(0.0).sqrt(),
        2.0 * (half - apart).max(0.0).sqrt(),
      ],
      angle: 0.5 * (2.0 * xy).atan2(xx - yy),
    })
  }

  /// How far, on average, `points` lie from this ellipse, as a share of its
  /// radius in their direction.
  pub(super) fn stray(&self, points: &[AnnotationPoint]) -> f64 {
    if self.radii[1] <= 0.0 {
      return f64::INFINITY;
    }
    let (sin, cos) = self.angle.sin_cos();
    let total: f64 = points
      .iter()
      .map(|point| {
        let (dx, dy) = (point.x - self.centre.x, point.y - self.centre.y);
        let along = (dx * cos + dy * sin) / self.radii[0];
        let across = (dy * cos - dx * sin) / self.radii[1];
        (along.hypot(across) - 1.0).abs()
      })
      .sum();
    total / points.len() as f64
  }

  /// `count` points round the ellipse, closed back to the first.
  pub(crate) fn outline(&self, count: usize) -> Vec<AnnotationPoint> {
    let (sin, cos) = self.angle.sin_cos();
    (0..=count)
      .map(|step| {
        let t = std::f64::consts::TAU * step as f64 / count as f64;
        let (along, across) = (self.radii[0] * t.cos(), self.radii[1] * t.sin());
        AnnotationPoint {
          x: self.centre.x + along * cos - across * sin,
          y: self.centre.y + along * sin + across * cos,
        }
      })
      .collect()
  }
}
