// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's prepared record, and the distance that picks it.
//!
//! The record carries, in the pixels it was prepared in: the picture's
//! middle in `a`, the middle of its right side in `b` and of its bottom side
//! in `c` as it lies before its tilt, so its half width and half height are
//! the lengths from `a` and its flat turn is their direction; its full width
//! in `width`, and its corner radius in `rounding`, all as this frame draws
//! it; whether it is mirrored in `head`; and its turn in depth, in radians,
//! in `start_head.c`. The compositor fills the rest of `start_head` with
//! where its picture was rasterised, and `end_head` with where its shadow
//! was.
//!
//! It arrives the way a counter does, growing out of its middle, so
//! `reveal.scale` shrinks it.

use super::ImageTilt;
use crate::geometry::{add, length, scale, subtract, ArrowGeometry};
use crate::reveal::AnnotationReveal;
use crate::text::geometry::rounded_box_distance;
use crate::AnnotationPoint;

/// `center` the picture's middle, `right` and `bottom` the middles of its
/// right and bottom sides as it lies before its tilt, `radius` its corner
/// radius as a percentage of its shorter side, `flip` whether it is
/// mirrored, and `tilt` its turn in depth in radians.
pub fn prepare_image(
  center: [f32; 2],
  right: [f32; 2],
  bottom: [f32; 2],
  radius: f32,
  flip: bool,
  tilt: [f32; 2],
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let grow = reveal.scale.clamp(0.0, 1.0);
  let across = subtract(right, center);
  let down = subtract(bottom, center);
  let (half_width, half_height) = (length(across), length(down));
  let share = if radius.is_finite() {
    radius.clamp(0.0, 50.0) / 100.0
  } else {
    0.0
  };
  let mut geometry = ArrowGeometry {
    a: center,
    b: add(center, scale(across, grow)),
    c: add(center, scale(down, grow)),
    width: 2.0 * half_width * grow,
    low: 0.0,
    high: 1.0,
    rounding: 2.0 * half_width.min(half_height) * share * grow,
    head: u32::from(flip),
    ..Default::default()
  };
  geometry.start_head.c = tilt;
  geometry
}

/// How far a point falls outside a prepared image: zero or less anywhere on
/// its picture's box as its tilt draws it, so a press on a transparent
/// corner still picks it.
pub fn image_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let across = subtract(geometry.b, geometry.a);
  let down = subtract(geometry.c, geometry.a);
  let (half_width, half_height) = (length(across), length(down));
  if half_width <= 0.0 || half_height <= 0.0 {
    return f32::INFINITY;
  }
  let offset = subtract(point, geometry.a);
  let drawn = AnnotationPoint {
    x: f64::from((offset[0] * across[0] + offset[1] * across[1]) / half_width),
    y: f64::from((offset[0] * down[0] + offset[1] * down[1]) / half_height),
  };
  let longer = f64::from(2.0 * half_width.max(half_height));
  let Some(local) = ImageTilt::from_radians(geometry.start_head.c).untilted(longer, drawn) else {
    return f32::INFINITY;
  };
  rounded_box_distance(
    [local.x as f32, local.y as f32],
    [-half_width, -half_height],
    [half_width, half_height],
    geometry.rounding,
  )
}
