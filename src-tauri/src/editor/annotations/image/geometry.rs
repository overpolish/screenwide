// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's prepared record, and the distance that picks it.
//!
//! The record carries, in the pixels it was prepared in: the picture's
//! middle in `a`, the middle of its right side in `b` and of its bottom side
//! in `c`, so its half width and half height are the lengths from `a` and
//! its turn is their direction; its full width in `width`, and its corner
//! radius in `rounding`, all as this frame draws it; whether it is mirrored in
//! `head`; and its turning grip, above its top side, in `end_head.c`, for
//! the chrome. The compositor fills `start_head` with where its picture was
//! rasterised, and replaces `end_head` with where its shadow was.
//!
//! It arrives the way a counter does, growing out of its middle, so
//! `reveal.scale` shrinks everything but the grip, which the chrome only
//! draws on an image at rest.

use crate::editor::annotations::geometry::{add, length, scale, subtract, ArrowGeometry};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::rounded_box_distance;

/// How far above the picture's top side its turning grip stands, in the
/// pixels it was prepared in: display points, where the chrome places it.
const TURN_GRIP_REACH: f32 = 22.0;

/// `center` the picture's middle, `right` and `bottom` the middles of its
/// right and bottom sides, `radius` its corner radius as a percentage of its
/// shorter side, and `flip` whether it is mirrored.
pub(crate) fn prepare_image(
  center: [f32; 2],
  right: [f32; 2],
  bottom: [f32; 2],
  radius: f32,
  flip: bool,
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
  if half_height > 0.0 {
    let up = scale(down, -1.0 / half_height);
    geometry.end_head.c = add(center, scale(up, half_height + TURN_GRIP_REACH));
  }
  geometry
}

/// How far a point falls outside a prepared image: zero or less anywhere on
/// its picture's box, so a press on a transparent corner still picks it.
pub(crate) fn image_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let across = subtract(geometry.b, geometry.a);
  let down = subtract(geometry.c, geometry.a);
  let (half_width, half_height) = (length(across), length(down));
  if half_width <= 0.0 || half_height <= 0.0 {
    return f32::INFINITY;
  }
  let offset = subtract(point, geometry.a);
  let local = [
    (offset[0] * across[0] + offset[1] * across[1]) / half_width,
    (offset[0] * down[0] + offset[1] * down[1]) / half_height,
  ];
  rounded_box_distance(
    local,
    [-half_width, -half_height],
    [half_width, half_height],
    geometry.rounding,
  )
}
