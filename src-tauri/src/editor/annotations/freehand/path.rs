// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fitting a stroke's points to a smooth chain of quadratic Béziers.
//!
//! The points are first thinned to the corners the line actually turns at
//! (Ramer-Douglas-Peucker). Each corner is then rounded by a curve that
//! leaves the leg into it and joins the leg out of it, bent by the corner
//! itself, and the legs between are drawn straight. Every curve leaves in the
//! direction the last one arrived in, so the line is smooth wherever it
//! bends, and it still starts and ends exactly where the hand did.
//!
//! A drawn stroke is thinned by under a pixel and its corners rounded by only
//! a few, which keeps the hand - a drawn L stays an L - and loses only the
//! sensor's jitter. A smoothed one is thinned by a share of its own size and
//! every corner rounded across half of each leg, which keeps only its shape.
//!
//! The chain is `a, b, c, b, c, ...`: its first point, then each curve's
//! control point and end. A straight leg is a curve bent by its own middle.
//! One point is a dot, drawn as a curve that goes nowhere.

use crate::editor::annotations::AnnotationPoint;

/// How far a drawn stroke may be thinned from what the hand drew, and how far
/// back along each leg its corners are rounded from, in source pixels.
const DRAWN_TOLERANCE: f32 = 0.6;
const DRAWN_ROUNDING: f32 = 3.0;

/// How far a smoothed stroke may stray from what the hand drew, as a share of
/// the stroke's longer side, and never less than a few pixels.
const SMOOTH_SHARE: f32 = 0.025;
const SMOOTH_LEAST: f32 = 2.0;

/// The most curves one stroke is drawn with. Each is measured at every pixel
/// near it, so a stroke with more is thinned harder until it fits. The twin
/// of `MaxPoints` in the macOS chrome's `annotation_draw_distance`, which
/// holds a whole chain.
pub(crate) const MAX_CURVES: usize = 256;

/// `points` with every point dropped that lies within `tolerance` of the line
/// between the points kept either side of it.
pub(crate) fn simplify(points: &[[f32; 2]], tolerance: f32) -> Vec<[f32; 2]> {
  if points.len() < 3 {
    return points.to_vec();
  }
  let mut keep = vec![false; points.len()];
  keep[0] = true;
  keep[points.len() - 1] = true;
  let mut spans = vec![(0, points.len() - 1)];
  while let Some((first, last)) = spans.pop() {
    let (mut farthest, mut index) = (0.0_f32, first);
    for (at, point) in points.iter().enumerate().take(last).skip(first + 1) {
      let off = segment_distance(*point, points[first], points[last]);
      if off > farthest {
        (farthest, index) = (off, at);
      }
    }
    if farthest > tolerance {
      keep[index] = true;
      spans.push((first, index));
      spans.push((index, last));
    }
  }
  points
    .iter()
    .zip(keep)
    .filter_map(|(point, kept)| kept.then_some(*point))
    .collect()
}

fn segment_distance(point: [f32; 2], from: [f32; 2], to: [f32; 2]) -> f32 {
  let leg = [to[0] - from[0], to[1] - from[1]];
  let off = [point[0] - from[0], point[1] - from[1]];
  let squared = leg[0] * leg[0] + leg[1] * leg[1];
  let along = if squared > 0.0 {
    ((off[0] * leg[0] + off[1] * leg[1]) / squared).clamp(0.0, 1.0)
  } else {
    0.0
  };
  (off[0] - leg[0] * along).hypot(off[1] - leg[1] * along)
}

fn middle(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
  [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Where the leg from `corner` to `toward` is left, rounding the corner by
/// `rounding`: never past its middle, which the next corner may need.
fn along_leg(corner: [f32; 2], toward: [f32; 2], rounding: f32) -> [f32; 2] {
  let leg = [toward[0] - corner[0], toward[1] - corner[1]];
  let length = leg[0].hypot(leg[1]);
  let share = if length > 0.0 {
    (rounding / length).min(0.5)
  } else {
    0.0
  };
  [corner[0] + leg[0] * share, corner[1] + leg[1] * share]
}

/// Appends a straight curve from the chain's last point to `to`, unless it
/// is already there.
fn straight(out: &mut Vec<[f32; 2]>, to: [f32; 2]) {
  let from = out[out.len() - 1];
  if (to[0] - from[0]).hypot(to[1] - from[1]) > 1e-3 {
    out.push(middle(from, to));
    out.push(to);
  }
}

/// The chain of curves through `points`, each corner rounded back along its
/// legs by `rounding`, or by half of each leg where that is less. A loop -
/// points that end where they began - rounds that corner too, so its ends
/// meet as smoothly as any other corner rather than in a point.
pub(crate) fn chain(points: &[[f32; 2]], rounding: f32) -> Vec<[f32; 2]> {
  match points {
    [] => Vec::new(),
    [only] => vec![*only; 3],
    _ => {
      let last = points.len() - 1;
      let closed =
        last >= 3 && (points[last][0] - points[0][0]).hypot(points[last][1] - points[0][1]) < 1e-3;
      let first = if closed {
        along_leg(points[0], points[1], rounding)
      } else {
        points[0]
      };
      let mut out = vec![first];
      for index in 1..last {
        let corner = points[index];
        straight(&mut out, along_leg(corner, points[index - 1], rounding));
        out.push(corner);
        out.push(along_leg(corner, points[index + 1], rounding));
      }
      if closed {
        let corner = points[0];
        straight(&mut out, along_leg(corner, points[last - 1], rounding));
        out.push(corner);
        out.push(first);
      } else {
        straight(&mut out, points[last]);
      }
      out
    }
  }
}

/// How many curves a chain holds.
pub(crate) fn curves(chain: &[[f32; 2]]) -> usize {
  chain.len().saturating_sub(1) / 2
}

/// The chain a stroke is drawn with, in source pixels.
pub(crate) fn fitted(points: &[AnnotationPoint], smooth: bool) -> Vec<[f32; 2]> {
  let raw: Vec<[f32; 2]> = points
    .iter()
    .map(|point| [point.x as f32, point.y as f32])
    .collect();
  let (low, high) = super::model::bounds(points);
  let longest = ((high.x - low.x).max(high.y - low.y)) as f32;
  let (mut tolerance, rounding) = if smooth {
    ((longest * SMOOTH_SHARE).max(SMOOTH_LEAST), f32::INFINITY)
  } else {
    (DRAWN_TOLERANCE, DRAWN_ROUNDING)
  };
  loop {
    let fitted = chain(&simplify(&raw, tolerance), rounding);
    if curves(&fitted) <= MAX_CURVES {
      return fitted;
    }
    tolerance *= 2.0;
  }
}

/// The squared distance from `point` to the curve `a, b, c` at `t`.
fn squared(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2], t: f32) -> f32 {
  let on = crate::editor::annotations::geometry::bezier(a, b, c, t);
  let off = [on[0] - point[0], on[1] - point[1]];
  off[0] * off[0] + off[1] * off[1]
}

/// How far `point` is from the curve `a, b, c`: sampled, then narrowed down
/// by golden-section search round the nearest sample, as the shaders do.
pub(crate) fn curve_distance(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
  const SAMPLES: usize = 16;
  let step = 1.0 / SAMPLES as f32;
  let mut best = 0.0;
  let mut best_squared = f32::INFINITY;
  for index in 0..=SAMPLES {
    let t = index as f32 * step;
    let at = squared(point, a, b, c, t);
    if at < best_squared {
      (best, best_squared) = (t, at);
    }
  }
  let (mut left, mut right) = ((best - step).max(0.0), (best + step).min(1.0));
  const GOLDEN: f32 = 0.618_034;
  for _ in 0..12 {
    let inner_left = right - (right - left) * GOLDEN;
    let inner_right = left + (right - left) * GOLDEN;
    if squared(point, a, b, c, inner_left) < squared(point, a, b, c, inner_right) {
      right = inner_right;
    } else {
      left = inner_left;
    }
  }
  best_squared
    .min(squared(point, a, b, c, (left + right) * 0.5))
    .sqrt()
}

/// How far `point` is from the chain's line, `f32::INFINITY` for an empty
/// chain.
pub(crate) fn chain_distance(point: [f32; 2], chain: &[[f32; 2]]) -> f32 {
  (0..curves(chain))
    .map(|curve| {
      let at = curve * 2;
      curve_distance(point, chain[at], chain[at + 1], chain[at + 2])
    })
    .fold(f32::INFINITY, f32::min)
}
