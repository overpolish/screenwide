// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a counter's tail tip lands while the counter is being turned.
//!
//! Turning a tail cannot reach an element edge the way a free point can: the
//! tip rides a circle of [`COUNTER_TAIL_REACH`] radii around the disc, so the
//! only positions on offer are where that circle crosses the edge's line.
//! Each crossing is one angle, and the aim takes the nearest of them within
//! the same eight screen points every other snap reaches - a distance along
//! the circle, which its radius turns into an angle.

use super::{AnnotationPoint, SnapBounds};
use crate::editor::annotations::counter::silhouette::COUNTER_TAIL_REACH;

/// The angle that lands the tail tip on a detected element's edge, the tip it
/// lands on, and the element it belongs to.
///
/// `angle` is the aim the hand gives, before Shift has quantised anything.
/// Only elements whose padded rectangle reaches the circle are looked at: a
/// 5K frame holds thousands of them and all but a handful are nowhere near
/// the counter.
pub(crate) fn snap_tail(
  center: AnnotationPoint,
  radius: f64,
  angle: f64,
  bounds: &[SnapBounds],
  threshold: f64,
) -> Option<(f64, AnnotationPoint, SnapBounds)> {
  let reach = radius * COUNTER_TAIL_REACH;
  if !reach.is_finite() || reach <= 0.0 {
    return None;
  }
  // Eight screen points along the circle, as the angle they subtend.
  let allowed = threshold / reach;
  let span = reach + threshold;
  let circle = SnapBounds {
    x: center.x - span,
    y: center.y - span,
    width: span * 2.0,
    height: span * 2.0,
  };
  let mut best: Option<(f64, f64, AnnotationPoint, SnapBounds)> = None;
  for rect in bounds {
    if !circle.intersects(*rect) {
      continue;
    }
    let mut consider = |tip: AnnotationPoint| {
      let candidate = (tip.y - center.y).atan2(tip.x - center.x);
      let travel = folded(candidate - angle).abs();
      if travel > allowed {
        return;
      }
      if best.is_none_or(|(chosen, ..)| travel < chosen) {
        best = Some((travel, candidate, tip, *rect));
      }
    };
    // The span each crossing has to fall inside is the edge itself grown by
    // the threshold, the way a tip riding an edge reaches its corners from
    // outside the rectangle.
    for edge in [rect.x, rect.right()] {
      let Some(offset) = crossing(edge - center.x, reach) else {
        continue;
      };
      for y in [center.y - offset, center.y + offset] {
        if y >= rect.y - threshold && y <= rect.bottom() + threshold {
          consider(AnnotationPoint { x: edge, y });
        }
      }
    }
    for edge in [rect.y, rect.bottom()] {
      let Some(offset) = crossing(edge - center.y, reach) else {
        continue;
      };
      for x in [center.x - offset, center.x + offset] {
        if x >= rect.x - threshold && x <= rect.right() + threshold {
          consider(AnnotationPoint { x, y: edge });
        }
      }
    }
  }
  best.map(|(_, angle, tip, rect)| (angle, tip, rect))
}

/// How far along an edge's line the circle crosses it, either side of the
/// foot of the perpendicular. `None` where the line misses the circle.
fn crossing(distance: f64, reach: f64) -> Option<f64> {
  let square = reach * reach - distance * distance;
  (square >= 0.0).then(|| square.sqrt())
}

/// A difference between two angles, folded into a half turn either way, so
/// the short way round a crossing near east is never measured the long way.
fn folded(delta: f64) -> f64 {
  let turn = std::f64::consts::TAU;
  let wrapped = delta.rem_euclid(turn);
  if wrapped > turn / 2.0 {
    wrapped - turn
  } else {
    wrapped
  }
}
