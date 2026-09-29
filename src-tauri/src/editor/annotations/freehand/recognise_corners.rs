// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a stroke turns, and the straight lines it runs between.
//!
//! The stroke is walked at even steps and simplified to the fewest points
//! that stay within a small share of its size of it. Those points are where
//! it turns: a hand's corner however obtuse, and the steps a curve is broken
//! into. The two are told apart by the legs between them. A leg the hand drew
//! straight lies close along one line; a piece of a curve bows away from its
//! line by a far larger share of its length, since the simplifying let it
//! bow as far as it could. A stroke made only of straight legs is drawn again
//! as those lines, each fitted through the points the hand drew along it and
//! meeting its neighbours where the two lines cross.

use super::{along, distance, path_length, segment_distance};
use crate::editor::annotations::AnnotationPoint;

/// A straight leg's line.
#[path = "recognise_line.rs"]
mod line;
use line::{straight_line, Line};

/// How many even steps the stroke is walked in.
const STEPS: usize = 128;
/// How far the simplified stroke may lie from the stroke, as a share of the
/// diagonal of the box round it.
const SIMPLIFY_SHARE: f64 = 0.04;
/// The least turn, in degrees, a simplified point makes to be kept: less is
/// a straight leg the simplifying split for its wobble.
const LEAST_TURN: f64 = 25.0;
/// The shortest leg, as a share of the stroke, that is a side of its own;
/// a shorter one is a corner the hand rounded. A star's legs are a tenth of
/// it.
const SHORT_SHARE: f64 = 0.06;
/// How many steps either side of a corner a hand may round it by, left out
/// when a leg is judged straight.
const ROUNDED: usize = 3;

/// A stroke walked at even steps: round the loop back to its first where
/// `closed`, which does not repeat it, and otherwise from end to end.
pub(super) struct Walk {
  pub(super) even: Vec<AnnotationPoint>,
  closed: bool,
}

pub(super) fn walk(points: &[AnnotationPoint], closed: bool) -> Walk {
  let mut path = points.to_vec();
  if closed {
    path.push(points[0]);
  }
  let length = path_length(&path);
  let count = if closed { STEPS } else { STEPS + 1 };
  let even = (0..count)
    .map(|step| along(&path, length * step as f64 / STEPS as f64))
    .collect();
  Walk { even, closed }
}

impl Walk {
  /// The steps the stroke turns at: its simplified points, less those that
  /// barely turn. An open stroke's ends are always among them.
  pub(super) fn turns(&self) -> Vec<usize> {
    let mut turns = self.simplified();
    // Drop the points that barely turn, one at a time from the least, since
    // dropping one changes its neighbours' turns; and join the two ends of a
    // leg far shorter than the stroke, a corner the hand rounded that the
    // simplifying split in two.
    let least = if self.closed { 3 } else { 2 };
    while turns.len() > least && (self.drop_slack(&mut turns) || self.join_short(&mut turns)) {}
    turns
  }

  /// The points a curve is drawn through, for a smooth line to pass by in
  /// turn: the simplified points, with a leg far shorter than the stroke
  /// joined into one point. A mouse or a trackpad jerks the line aside for a
  /// moment; that leaves a short leg, and joining it leaves the curve the
  /// hand meant rather than a kink in it.
  pub(super) fn curve(&self) -> Vec<AnnotationPoint> {
    let mut turns = self.simplified();
    let least = if self.closed { 3 } else { 2 };
    while turns.len() > least && self.join_short(&mut turns) {}
    turns.iter().map(|&step| self.even[step]).collect()
  }

  /// The steps of the fewest points that stay within [`SIMPLIFY_SHARE`] of
  /// the stroke's size of it.
  fn simplified(&self) -> Vec<usize> {
    let (low, high) = super::super::model::bounds(&self.even);
    let tolerance = SIMPLIFY_SHARE * distance(low, high);
    if !self.closed {
      return simplify(&self.even, tolerance);
    }
    // A loop is cut in two at the step farthest from its first, and each
    // half simplified between them.
    let count = self.even.len();
    let far = (0..count)
      .max_by(|&a, &b| {
        distance(self.even[0], self.even[a]).total_cmp(&distance(self.even[0], self.even[b]))
      })
      .unwrap_or(0);
    let mut ring = self.even.clone();
    ring.push(self.even[0]);
    let mut turns = simplify(&ring[..=far], tolerance);
    turns.extend(
      simplify(&ring[far..], tolerance)
        .into_iter()
        .skip(1)
        .map(|step| far + step),
    );
    turns.pop();
    turns
  }

  /// Drops the simplified point that turns least, where it turns less than
  /// [`LEAST_TURN`]. An open stroke's ends stay.
  fn drop_slack(&self, turns: &mut Vec<usize>) -> bool {
    let slack = (0..turns.len())
      .filter(|&at| self.closed || (at > 0 && at + 1 < turns.len()))
      .map(|at| (at, self.turn_at(turns, at)))
      .min_by(|a, b| a.1.total_cmp(&b.1))
      .filter(|(_, turn)| *turn < LEAST_TURN);
    if let Some((at, _)) = slack {
      turns.remove(at);
    }
    slack.is_some()
  }

  /// Joins the ends of the shortest leg into one point halfway along it,
  /// where the leg is shorter than [`SHORT_SHARE`] of the stroke. A leg that
  /// ends an open stroke keeps that end, and loses its other instead.
  fn join_short(&self, turns: &mut Vec<usize>) -> bool {
    let count = self.even.len();
    let legs = if self.closed {
      turns.len()
    } else {
      turns.len() - 1
    };
    let short = (0..legs)
      .map(|leg| {
        (
          leg,
          (turns[(leg + 1) % turns.len()] + count - turns[leg]) % count,
        )
      })
      .min_by_key(|(_, steps)| *steps)
      .filter(|(_, steps)| (*steps as f64) < SHORT_SHARE * count as f64);
    let Some((leg, steps)) = short else {
      return false;
    };
    let next = (leg + 1) % turns.len();
    if !self.closed && (leg == 0 || next + 1 == turns.len()) {
      turns.remove(if leg == 0 { next } else { leg });
    } else {
      turns[leg] = (turns[leg] + steps / 2) % count;
      turns.remove(next);
      turns.sort_unstable();
    }
    true
  }

  /// How far, in degrees, the simplified stroke turns at its `at`th point.
  fn turn_at(&self, turns: &[usize], at: usize) -> f64 {
    let count = turns.len();
    let before = self.even[turns[(at + count - 1) % count]];
    let here = self.even[turns[at]];
    let after = self.even[turns[(at + 1) % count]];
    let turn =
      (after.y - here.y).atan2(after.x - here.x) - (here.y - before.y).atan2(here.x - before.x);
    ((turn + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI)
      .abs()
      .to_degrees()
  }

  /// The stroke as the straight lines it runs between its `turns`: the points
  /// where neighbouring legs' lines cross, closed back to the first for a
  /// loop, and set on its first and last legs at an open stroke's ends.
  /// `None` where any leg bows like a piece of a curve.
  pub(super) fn straightened(&self, turns: &[usize]) -> Option<Vec<AnnotationPoint>> {
    let count = self.even.len();
    let legs = if self.closed {
      turns.len()
    } else {
      turns.len() - 1
    };
    let lines = (0..legs)
      .map(|leg| {
        let (from, to) = (turns[leg], turns[(leg + 1) % turns.len()]);
        let steps = (to + count - from) % count;
        // A leg too short to leave out its rounded ends is judged whole.
        let trim = if steps > 2 * ROUNDED + 1 { ROUNDED } else { 0 };
        let points: Vec<AnnotationPoint> = (trim..=steps - trim)
          .map(|offset| self.even[(from + offset) % count])
          .collect();
        straight_line(&points)
      })
      .collect::<Option<Vec<_>>>()?;
    // Two legs that nearly double back cross far from the corner the hand
    // turned, if at all; that corner then stays where the hand turned it.
    let meet = |at: usize, a: &Line, b: &Line| {
      let turned = self.even[at];
      a.cross(b)
        .filter(|point| distance(*point, turned) <= a.length.min(b.length) / 2.0)
        .unwrap_or(turned)
    };
    if self.closed {
      let mut vertices: Vec<AnnotationPoint> = (0..lines.len())
        .map(|leg| {
          meet(
            turns[leg],
            &lines[(leg + lines.len() - 1) % lines.len()],
            &lines[leg],
          )
        })
        .collect();
      vertices.push(vertices[0]);
      return Some(vertices);
    }
    let mut vertices = vec![lines[0].foot(self.even[0])];
    for corner in 1..turns.len() - 1 {
      vertices.push(meet(turns[corner], &lines[corner - 1], &lines[corner]));
    }
    vertices.push(lines[lines.len() - 1].foot(self.even[count - 1]));
    Some(vertices)
  }
}

/// The indices of `points` a Ramer-Douglas-Peucker simplification keeps.
fn simplify(points: &[AnnotationPoint], tolerance: f64) -> Vec<usize> {
  let last = points.len() - 1;
  let mut kept = vec![0, last];
  let mut spans = vec![(0, last)];
  while let Some((from, to)) = spans.pop() {
    let farthest = (from + 1..to)
      .map(|index| {
        (
          index,
          segment_distance(points[index], points[from], points[to]),
        )
      })
      .max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((index, _)) = farthest.filter(|(_, away)| *away > tolerance) {
      kept.push(index);
      spans.push((from, index));
      spans.push((index, to));
    }
  }
  kept.sort_unstable();
  kept.dedup();
  kept
}
