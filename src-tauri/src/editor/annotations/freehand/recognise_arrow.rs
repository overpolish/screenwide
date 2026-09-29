// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading an open stroke as an arrow with its head drawn on.

use super::{corners, distance, fitted, path_length, Recognised};
use crate::editor::annotations::AnnotationPoint;

/// How far a head may reach from the tip, as a share of the shaft.
const HOOK_REACH: f64 = 0.55;
/// The least a head runs, as a share of the shaft, so the jitter of a hand
/// coming to rest is not read as one.
const HOOK_SHARE: f64 = 0.1;
/// The most a head runs, as a share of the shaft: a hand that doubles back
/// and forth draws a head in more than one go, but a run longer than this is
/// more of the drawing, not its head.
const HEAD_SHARE: f64 = 1.5;
/// The least a head runs, in screen points.
const HOOK_LEAST: f64 = 8.0;
/// How far back along the shaft a head must turn, as the cosine of the angle
/// between them: a barb sweeps back past square, where a foot stands square.
const HOOK_BACK: f64 = -0.3;

/// A line with a head at its end: a shaft out to its tip, then a short run
/// that turns back from the tip, as a hand draws an arrowhead without
/// lifting. The tip is where the stroke first turns, and failing that the
/// point farthest from where it began, which is where a head drawn past the
/// tip leaves the shaft.
pub(super) fn hooked_arrow(points: &[AnnotationPoint], unit: f64) -> Option<Recognised> {
  let start = points[0];
  let walk = corners::walk(points, false);
  let turns = walk.turns();
  // The walk's steps are even along the stroke, so its first turn is that
  // far along it: a head drawn back past the tip passes near it again,
  // which nearness alone would not tell apart. The tip is then the point
  // farthest from the start within a couple of steps of it.
  let first_turn = (turns.len() > 2).then(|| {
    let length = path_length(points);
    let step = length / (walk.even.len() - 1) as f64;
    let at = step * turns[1] as f64;
    let mut walked = 0.0;
    let along: Vec<f64> = std::iter::once(0.0)
      .chain((1..points.len()).map(|index| {
        walked += distance(points[index - 1], points[index]);
        walked
      }))
      .collect();
    (0..points.len())
      .filter(|&index| (along[index] - at).abs() <= 2.0 * step)
      .max_by(|&a, &b| distance(start, points[a]).total_cmp(&distance(start, points[b])))
      .unwrap_or(points.len() - 1)
  });
  let farthest = (0..points.len())
    .max_by(|&a, &b| distance(start, points[a]).total_cmp(&distance(start, points[b])));
  [first_turn, farthest]
    .into_iter()
    .flatten()
    .find_map(|tip| head_at(points, tip, unit))
}

/// The arrow whose shaft runs from the stroke's start to `points[tip]`, where
/// the rest of the stroke is a head there: short beside the shaft, near the
/// tip, and turning back along it.
fn head_at(points: &[AnnotationPoint], tip: usize, unit: f64) -> Option<Recognised> {
  let (start, at) = (points[0], points[tip]);
  let shaft = distance(start, at);
  let head = &points[tip..];
  let run = path_length(head);
  if head.len() < 2 || run < (HOOK_SHARE * shaft).max(HOOK_LEAST * unit) || run > HEAD_SHARE * shaft
  {
    return None;
  }
  if head
    .iter()
    .any(|point| distance(at, *point) > HOOK_REACH * shaft)
  {
    return None;
  }
  let along = [(at.x - start.x) / shaft, (at.y - start.y) / shaft];
  let turns_back = head.iter().any(|point| {
    let away = distance(at, *point);
    away >= HOOK_LEAST * unit
      && (point.x - at.x) * along[0] + (point.y - at.y) * along[1] <= HOOK_BACK * away
  });
  if !turns_back {
    return None;
  }
  let (start, control, end) = fitted(&points[..=tip], unit)?;
  Some(Recognised::Arrow {
    start,
    control,
    end,
    head: true,
  })
}
