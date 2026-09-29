// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading a closed stroke.
//!
//! A loop drawn with straight sides is those lines - a triangle, a slanted
//! box - or a box shape where it has four sides that none of them leans far
//! from upright or across. A loop that follows an ellipse is a round shape
//! wherever one will do: a circle where it is nearly one, a pill where it
//! lies upright or across, and only where it leans is it the ellipse itself,
//! turned as it was drawn. A box whose sides bow, as a hand's box with round
//! corners does, is still a box where it lies along the box round it.
//! Anything else is left to be smoothed.

use super::corners::walk;
use super::ellipse::Ellipse;
use super::{distance, path_length, Recognised, SMALLEST};
use crate::editor::annotations::freehand::model::bounds;
use crate::editor::annotations::AnnotationPoint;

/// How far, in degrees, a box's sides may lean from upright or across for
/// the box to be taken upright. A hand's box leans its sides by ten or so; a
/// box drawn turned, or a slanted one, by far more.
const UPRIGHT_TILT: f64 = 20.0;
/// How far on average a loop whose sides bow may lie from the box round it,
/// or from its ellipse, as a share of that box or ellipse, to be taken for
/// whichever it lies nearer. A hand's circles and ovals lie two to eight
/// percent from their ellipse, its bowed boxes about as far from their box,
/// and each lies further from the other.
const OUTLINE_STRAY: f64 = 0.12;
/// How near its two radii must come, the shorter as a share of the longer,
/// for an ellipse to be taken for a circle. A hand's circle comes out within
/// a fifth.
const CIRCLE_SHARE: f64 = 0.75;
/// How far, in degrees, an ellipse's long radius may lean from upright or
/// across for a pill to stand in for it.
const PILL_TILT: f64 = 20.0;

/// A closed stroke read as its straight lines, an upright box, or an
/// ellipse; `None` where it is too thin to read, or none of them fits.
pub(super) fn outline(points: &[AnnotationPoint], unit: f64) -> Option<Recognised> {
  let points = &points[..=closing(points)];
  let (low, high) = bounds(points);
  if (high.x - low.x).min(high.y - low.y) < SMALLEST * unit / 2.0 {
    return None;
  }
  let walk = walk(points, true);
  let turns = walk.turns();
  if let Some(vertices) = walk.straightened(&turns) {
    let upright = turns.len() == 4
      && (0..4).all(|side| {
        let (a, b) = (vertices[side], vertices[side + 1]);
        let angle = (b.y - a.y).atan2(b.x - a.x).to_degrees().rem_euclid(90.0);
        angle.min(90.0 - angle) <= UPRIGHT_TILT
      });
    return Some(if upright {
      Recognised::Box { low, high }
    } else {
      Recognised::Lines { points: vertices }
    });
  }
  let ellipse = Ellipse::enclosed_by(&walk.even);
  let round_stray = ellipse.map_or(f64::INFINITY, |ellipse| ellipse.stray(&walk.even));
  let box_stray = box_stray(&walk.even, low, high);
  if box_stray.min(round_stray) > OUTLINE_STRAY {
    return None;
  }
  Some(match ellipse.filter(|_| round_stray <= box_stray) {
    Some(ellipse) => round(ellipse),
    None => Recognised::Box { low, high },
  })
}

/// The round shape standing in for `ellipse`: a circle round its centre
/// where its radii nearly agree, a pill over the box round it where it lies
/// upright or across, and otherwise the ellipse itself.
fn round(ellipse: Ellipse) -> Recognised {
  let [long, short] = ellipse.radii;
  let centre = ellipse.centre;
  let reach = if short >= CIRCLE_SHARE * long {
    let radius = (long + short) / 2.0;
    [radius, radius]
  } else {
    let lean = ellipse.angle.to_degrees().rem_euclid(90.0);
    if lean.min(90.0 - lean) > PILL_TILT {
      return Recognised::Ellipse(ellipse);
    }
    // Half the box round the ellipse, however little it leans.
    let (sin, cos) = ellipse.angle.sin_cos();
    [
      (long * long * cos * cos + short * short * sin * sin).sqrt(),
      (long * long * sin * sin + short * short * cos * cos).sqrt(),
    ]
  };
  Recognised::Round {
    low: AnnotationPoint {
      x: centre.x - reach[0],
      y: centre.y - reach[1],
    },
    high: AnnotationPoint {
      x: centre.x + reach[0],
      y: centre.y + reach[1],
    },
  }
}

/// How far on average `points` lie from the box `low` to `high`, as a share
/// of half the box.
fn box_stray(points: &[AnnotationPoint], low: AnnotationPoint, high: AnnotationPoint) -> f64 {
  let half = [(high.x - low.x) / 2.0, (high.y - low.y) / 2.0];
  let centre = [low.x + half[0], low.y + half[1]];
  let total: f64 = points
    .iter()
    .map(|point| {
      let x = ((point.x - centre[0]) / half[0]).abs();
      let y = ((point.y - centre[1]) / half[1]).abs();
      (x.max(y) - 1.0).abs()
    })
    .sum();
  total / points.len() as f64
}

/// Where a closed stroke comes back round to its start: the point nearest the
/// start in its second half. A hand that overshoots draws past it, and the
/// retraced run would otherwise read as more of the shape.
pub(super) fn closing(points: &[AnnotationPoint]) -> usize {
  let half = path_length(points) / 2.0;
  let mut walked = 0.0;
  let mut nearest = (points.len() - 1, f64::INFINITY);
  for index in 1..points.len() {
    walked += distance(points[index - 1], points[index]);
    let away = distance(points[0], points[index]);
    if walked >= half && away < nearest.1 {
      nearest = (index, away);
    }
  }
  nearest.0
}

/// How far an end may head on to reach the stroke, as a share of the longer
/// side of the box round it, to close it there.
const REACH_SHARE: f64 = 0.3;
/// How much of the stroke, from its start, an end may reach to close it.
const REACH_ALONG: f64 = 0.4;
/// How much of the stroke, back from its end, the end's heading is taken
/// across: enough to ride over a hand's last jitter.
const HEADING_SHARE: f64 = 0.08;

/// The loop a stroke makes where its end heads straight for the first part
/// of it, closed at the point the end would reach and trimmed of what ran
/// before that point; `None` where the end heads anywhere else.
///
/// A hand that stops short of where it began often stops short of the line
/// it began with rather than the point, and aims its end at it. A round
/// loop left open curves round past its start instead, and heads nowhere
/// near it.
pub(super) fn closed_by_reach(
  points: &[AnnotationPoint],
  size: f64,
) -> Option<Vec<AnnotationPoint>> {
  let mut walked = 0.0;
  let along: Vec<f64> = std::iter::once(0.0)
    .chain((1..points.len()).map(|index| {
      walked += distance(points[index - 1], points[index]);
      walked
    }))
    .collect();
  let length = walked;
  let end = points[points.len() - 1];
  let from = points[(0..points.len())
    .rev()
    .find(|&index| length - along[index] >= HEADING_SHARE * length)?];
  let run = distance(from, end);
  if run <= 0.0 {
    return None;
  }
  let heading = [(end.x - from.x) / run, (end.y - from.y) / run];
  let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
  // The nearest crossing ahead of the end with a leg of the first part.
  let (leg, reach) = (1..points.len())
    .take_while(|&index| along[index - 1] <= REACH_ALONG * length)
    .filter_map(|index| {
      let (a, b) = (points[index - 1], points[index]);
      let side = [b.x - a.x, b.y - a.y];
      let turn = cross(heading, side);
      if turn.abs() < f64::EPSILON {
        return None;
      }
      let offset = [a.x - end.x, a.y - end.y];
      let reach = cross(offset, side) / turn;
      let share = cross(offset, heading) / turn;
      ((0.0..=1.0).contains(&share) && reach > 0.0 && reach <= REACH_SHARE * size)
        .then_some((index, reach))
    })
    .min_by(|a, b| a.1.total_cmp(&b.1))?;
  let meet = AnnotationPoint {
    x: end.x + heading[0] * reach,
    y: end.y + heading[1] * reach,
  };
  let mut ring = vec![meet];
  ring.extend_from_slice(&points[leg..]);
  ring.push(meet);
  Some(ring)
}
