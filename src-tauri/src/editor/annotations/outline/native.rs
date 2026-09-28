// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shape's retained draw record.
//!
//! `p0` and `p2` are its box's corners, placed like any point, and `width`
//! its pen. `p1` is not a point: it carries the corner radius, as a
//! percentage of the box's shorter side, and the stroke's [`hand`], which no
//! placement touches.

use crate::editor::annotations::{AnnotationPoint, AnnotationStyle};

pub(crate) fn draw_points(
  start: AnnotationPoint,
  end: AnnotationPoint,
  seed: u32,
  style: &AnnotationStyle,
) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [radius(style) as f32, hand(style.hand_drawn, seed)],
    [end.x as f32, end.y as f32],
  ]
}

/// The corner radius a record carries: the style's percentage, held to the
/// range a box can be rounded over.
pub(crate) fn radius(style: &AnnotationStyle) -> f64 {
  if style.radius.is_finite() {
    style.radius.clamp(0.0, 50.0)
  } else {
    0.0
  }
}

/// Which stroke a shape is drawn with, in one number a float carries
/// exactly: zero for a clean stroke, or one more than the seed of a
/// hand-drawn one, folded to sixteen bits.
pub(crate) fn hand(hand_drawn: bool, seed: u32) -> f32 {
  if hand_drawn {
    1.0 + ((seed ^ (seed >> 16)) & 0xffff) as f32
  } else {
    0.0
  }
}
