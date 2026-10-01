// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shape's prepared record and the distance that picks it.
//!
//! The outline is a rounded box walked clockwise from the top side's left
//! end, four sides and four corners, and the pen's stroke is a window along
//! that walk with a round pen at each end. A square corner is a corner with
//! no length, whose one point faces every way between its two sides. The
//! record carries the box's corners in `a` and `b`, its corner radius in
//! `rounding`, the pen in `width`, and how much of the stroke this frame
//! draws, as shares of it, in `low` and `high`. `c` holds where on the walk
//! the stroke begins and how long it is. A clean stroke is exactly one lap,
//! so its ends meet; a hand-drawn one runs on and strays, as
//! [`super::wander`] decides, with `head` one.
//!
//! The distance measures every side and corner of the walk on its own and
//! keeps the nearest, so wherever the pen went is drawn, however far the
//! passes over one corner stray from each other. The shaders hold the only
//! other copies of [`shape_distance`]:
//! `gpu_compositor_macos_shader_source_annotation_shape.h` and
//! `annotation_shape.wgsl`.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::wander::{curl, hand_drawn, slope, wander};
use crate::editor::annotations::geometry::{add, length, scale, subtract, ArrowGeometry};
use crate::editor::annotations::reveal::AnnotationReveal;

/// The box between the two corners, in the space they are given in, rounded
/// by `radius` percent of its shorter side and drawn with a `width` pen.
/// `hand` is zero for a clean stroke, or one more than the seed of a
/// hand-drawn one, which is how the records carry both in one number.
pub(crate) fn prepare_shape(
  start: [f32; 2],
  end: [f32; 2],
  radius: f32,
  hand: f32,
  width: f32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let a = [start[0].min(end[0]), start[1].min(end[1])];
  let b = [start[0].max(end[0]), start[1].max(end[1])];
  let half = [(b[0] - a[0]) * 0.5, (b[1] - a[1]) * 0.5];
  let share = if radius.is_finite() {
    radius.clamp(0.0, 50.0) / 100.0
  } else {
    0.0
  };
  let rounding = 2.0 * half[0].min(half[1]).max(0.0) * share;
  let perimeter = perimeter(half, rounding);
  let window = |value: f32| {
    if value.is_finite() {
      value.clamp(0.0, 1.0)
    } else {
      0.0
    }
  };
  let mut geometry = ArrowGeometry {
    a,
    b,
    width: width.max(0.0),
    low: window(reveal.low),
    high: window(reveal.high),
    rounding,
    // A clean stroke starts halfway round the top-left corner, where a hand
    // would, and ends exactly there.
    c: [perimeter - FRAC_PI_2 * rounding * 0.5, perimeter],
    ..ArrowGeometry::default()
  };
  if hand.is_finite() && hand >= 1.0 && perimeter > 0.0 {
    let pen = geometry.width;
    hand_drawn(&mut geometry, hand as u32 - 1, perimeter, half, pen);
  }
  geometry
}

/// How far round the outline of a box `half` across each way, its corners
/// rounded by `rounding`.
pub(super) fn perimeter(half: [f32; 2], rounding: f32) -> f32 {
  4.0 * (half[0] + half[1] - 2.0 * rounding) + TAU * rounding
}

/// How many points along the pen's line each search for the nearest one
/// tries before its ends. Four settle it within a tenth of a pixel round
/// every curl a lead-in or a trail-off makes; the shaders try as many.
const TRIES: usize = 4;

/// The distance from `local` to the nearest point of the pen's line within
/// `span`, searching from `at`. `probe` reports, at a point of the line, how
/// far `local` is from it, how far `local` lies along the line's direction,
/// that direction's squared length, and how far `local` lies along the
/// line's curl: all a Newton step needs. Towards the centre of a curl the
/// nearest point flattens out and then splits in two, so a step is held to
/// twice what following the line alone would take, and one that lands
/// further away is halved back towards the best point so far. The span's
/// ends are measured too, so a tip is never missed.
fn nearest(span: (f32, f32), mut at: f32, probe: impl Fn(f32) -> [f32; 4]) -> f32 {
  let (mut best, mut best_at, mut shift) = (f32::INFINITY, at, 0.0);
  for _ in 0..TRIES {
    let [distance, gap, speed, pull] = probe(at);
    if distance < best {
      (best, best_at) = (distance, at);
      shift = gap / (speed - pull).max(speed * 0.5).max(1e-6);
    } else {
      shift *= 0.5;
    }
    at = (best_at + shift).clamp(span.0, span.1);
  }
  best
    .min(probe(at)[0])
    .min(probe(span.0)[0])
    .min(probe(span.1)[0])
}

/// Where the stroke is on the walk, and how much of it this frame draws.
struct Stroke<'a> {
  geometry: &'a ArrowGeometry,
  lap: f32,
  from: f32,
  to: f32,
}

impl Stroke<'_> {
  /// Each stretch of the stroke that passes over the piece of the walk
  /// starting `start` into it and running on for `run`: how far into the
  /// stroke the piece begins on that pass, and the part of the piece drawn.
  /// The stroke is at most a lap and a quarter, so it can pass over a piece
  /// on three laps at most.
  fn passes(&self, start: f32, run: f32) -> impl Iterator<Item = (f32, f32, f32)> + '_ {
    let base = (start - self.geometry.c[0]).rem_euclid(self.lap);
    [-1.0, 0.0, 1.0].into_iter().filter_map(move |turn: f32| {
      let offset = base + turn * self.lap;
      let (low, high) = ((self.from - offset).max(0.0), (self.to - offset).min(run));
      (low <= high).then_some((offset, low, high))
    })
  }

  /// How far `local` is from the pen's line along one side, from its
  /// `from` end along `toward` for `run`, `normal` pointing out. The search
  /// starts square to the side, so a line leaving the outline keeps the
  /// pen's width and a tip that flicks away ends in a true round pen.
  fn side(
    &self,
    local: [f32; 2],
    from: [f32; 2],
    toward: [f32; 2],
    normal: [f32; 2],
    start: f32,
    run: f32,
  ) -> f32 {
    let relative = subtract(local, from);
    let along = relative[0] * toward[0] + relative[1] * toward[1];
    let off = relative[0] * normal[0] + relative[1] * normal[1];
    self
      .passes(start, run)
      .map(|(offset, low, high)| {
        nearest((low, high), along.clamp(low, high), |at| {
          let beyond = off - wander(self.geometry, offset + at);
          let tilt = slope(self.geometry, offset + at);
          [
            (along - at).hypot(beyond),
            along - at + beyond * tilt,
            1.0 + tilt * tilt,
            beyond * curl(self.geometry, offset + at),
          ]
        })
      })
      .fold(f32::INFINITY, f32::min)
  }

  /// How far `local` is from the pen's line round one corner: a quarter
  /// turn about `centre` from `from_angle`, `start` into the walk, searched
  /// by the turn as a side is by its length. A square corner is passed at
  /// one moment, and faces every way between its sides.
  fn corner(
    &self,
    local: [f32; 2],
    centre: [f32; 2],
    from_angle: f32,
    rounding: f32,
    start: f32,
  ) -> f32 {
    let relative = subtract(local, centre);
    let reach = length(relative);
    let turned = relative[1].atan2(relative[0]) - from_angle;
    // Held to the half turn centred on the corner's own, so a point behind
    // it lands on the nearer of its two ends.
    let turned = turned - TAU * ((turned + 0.75 * PI) / TAU).floor();
    self
      .passes(start, FRAC_PI_2 * rounding)
      .map(|(offset, low, high)| {
        let probe = |at: f32| {
          let along = offset + at * rounding;
          let bend = rounding + wander(self.geometry, along);
          // A stroke pulled inside a corner further than it is rounded
          // needs no join: its two sides already cross there.
          if bend < 0.0 {
            return [f32::INFINITY, 0.0, 1.0, 0.0];
          }
          let lean = slope(self.geometry, along) * rounding;
          let swing = curl(self.geometry, along) * rounding * rounding;
          // `local` sits `out` beyond the line and `on` ahead of it. Per
          // unit of turn the line runs `lean` out and `bend` on, and curls
          // `swing - bend` out and `2 * lean` on.
          let (ahead, level) = (turned - at).sin_cos();
          let (out, on) = (reach * level - bend, reach * ahead);
          [
            out.hypot(on),
            out * lean + on * bend,
            lean * lean + bend * bend,
            out * (swing - bend) + on * 2.0 * lean,
          ]
        };
        // A square corner's wander is one number, so the clamp is exact.
        if rounding <= 0.0 {
          return probe(turned.clamp(0.0, FRAC_PI_2))[0];
        }
        let span = (low / rounding, high / rounding);
        nearest(span, turned.clamp(span.0, span.1), probe)
      })
      .fold(f32::INFINITY, f32::min)
  }
}

/// How far `point` falls outside the drawn stroke, in the space it was
/// prepared in: zero on its edge and negative inside it, so a press on the
/// pen's line picks the shape and a press inside the box does not.
pub(crate) fn shape_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let stroke = geometry.c[1];
  if geometry.high <= geometry.low || stroke <= 0.0 || stroke.is_nan() {
    return f32::INFINITY;
  }
  let centre = scale(add(geometry.a, geometry.b), 0.5);
  let half = scale(subtract(geometry.b, geometry.a), 0.5);
  let rounding = geometry.rounding.min(half[0]).min(half[1]).max(0.0);
  let local = subtract(point, centre);
  let walk = Stroke {
    geometry,
    lap: perimeter(half, rounding),
    from: geometry.low * stroke,
    to: geometry.high * stroke,
  };
  let inner = [half[0] - rounding, half[1] - rounding];
  let (across, down) = (2.0 * inner[0], 2.0 * inner[1]);
  // Clockwise from the top side's left end: each side, then the corner
  // after it.
  let sides = [
    ([-inner[0], -half[1]], [1.0, 0.0], [0.0, -1.0], across),
    ([half[0], -inner[1]], [0.0, 1.0], [1.0, 0.0], down),
    ([inner[0], half[1]], [-1.0, 0.0], [0.0, 1.0], across),
    ([-half[0], inner[1]], [0.0, -1.0], [-1.0, 0.0], down),
  ];
  let corners = [
    [inner[0], -inner[1]],
    [inner[0], inner[1]],
    [-inner[0], inner[1]],
    [-inner[0], -inner[1]],
  ];
  let mut start = 0.0;
  let mut nearest = f32::INFINITY;
  for turn in 0..4 {
    let (from, toward, normal, run) = sides[turn];
    nearest = nearest.min(walk.side(local, from, toward, normal, start, run));
    start += run;
    let from_angle = FRAC_PI_2 * (turn as f32 - 1.0);
    nearest = nearest.min(walk.corner(local, corners[turn], from_angle, rounding, start));
    start += FRAC_PI_2 * rounding;
  }
  nearest - geometry.width * 0.5
}

/// How far `point` falls outside the shape's stroke or the box it outlines:
/// what grabs a shape anywhere inside it, where the editor lets a press
/// there carry it rather than draw under it.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn shape_body_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  shape_distance(point, geometry).min(
    crate::editor::annotations::text::geometry::rounded_box_distance(
      point,
      geometry.a,
      geometry.b,
      geometry.rounding,
    ),
  )
}

#[cfg(test)]
mod tests;
