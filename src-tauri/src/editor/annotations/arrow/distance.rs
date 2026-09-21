// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How near a prepared arrow a point falls.
//!
//! Picking uses the same outline the shader draws, so a press lands on the
//! arrow exactly where the arrow looks like it is: the shaft is sampled along
//! the curve, and each head is its inner triangle grown back out by the
//! rounding.

use crate::editor::annotations::geometry::{
  add, bezier, distance, scale, subtract, ArrowGeometry, ArrowTriangle,
};

fn segment_distance(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
  let delta = subtract(end, start);
  let squared = delta[0] * delta[0] + delta[1] * delta[1];
  let t = if squared <= 0.0 {
    0.0
  } else {
    (((point[0] - start[0]) * delta[0] + (point[1] - start[1]) * delta[1]) / squared)
      .clamp(0.0, 1.0)
  };
  distance(point, add(start, scale(delta, t)))
}

/// How far a point is from a head's rounded triangle: zero inside it.
pub(crate) fn head_distance(point: [f32; 2], triangle: ArrowTriangle, rounding: f32) -> f32 {
  let ArrowTriangle { a, b, c } = triangle;
  let first = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
  let second = (c[0] - b[0]) * (point[1] - b[1]) - (c[1] - b[1]) * (point[0] - b[0]);
  let third = (a[0] - c[0]) * (point[1] - c[1]) - (a[1] - c[1]) * (point[0] - c[0]);
  let inside = (first >= 0.0 && second >= 0.0 && third >= 0.0)
    || (first <= 0.0 && second <= 0.0 && third <= 0.0);
  if inside {
    return 0.0;
  }
  (segment_distance(point, a, b)
    .min(segment_distance(point, b, c))
    .min(segment_distance(point, c, a))
    - rounding)
    .max(0.0)
}

/// How far a point is from the shaft, sampled finely enough for a fingertip.
pub(crate) fn shaft_distance(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
  const SAMPLES: u32 = 24;
  let mut best = f32::INFINITY;
  let mut previous = a;
  for sample in 1..=SAMPLES {
    let next = bezier(a, b, c, sample as f32 / SAMPLES as f32);
    best = best.min(segment_distance(point, previous, next));
    previous = next;
  }
  best
}

/// How far a point is from a prepared arrow's drawn shape: its shaft, and the
/// heads on it. Zero anywhere the arrow is painted, because the tolerance is
/// measured from the stroke's edge rather than its centreline. A press on a
/// head is a press on the arrow - it is the part of it the hand aims at.
pub(crate) fn prepared_arrow_distance(point: [f32; 2], arrow: &ArrowGeometry) -> f32 {
  let mut best = (shaft_distance(point, arrow.a, arrow.b, arrow.c) - arrow.width * 0.5).max(0.0);
  if arrow.head != 0 {
    best = best.min(head_distance(point, arrow.end_head, arrow.rounding));
  }
  if arrow.head == 2 {
    best = best.min(head_distance(point, arrow.start_head, arrow.rounding));
  }
  best
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn shaft_distance_follows_the_bend() {
    let distance = shaft_distance([100.0, 50.0], [0.0, 0.0], [100.0, 100.0], [200.0, 0.0]);
    assert!(distance < 1.0, "{distance}");
    assert!(shaft_distance([100.0, 0.0], [0.0, 0.0], [100.0, 100.0], [200.0, 0.0]) > 40.0);
  }
}
