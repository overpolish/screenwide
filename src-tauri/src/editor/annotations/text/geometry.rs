// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Draw-ready text box geometry, prepared once per annotation for both
//! backends, and the distance that picks it.
//!
//! The box is a rounded rectangle its text sits centred in. The pointer is a
//! tapered stroke from a round base hidden inside the box to a rounded tip,
//! blended into the box with a smooth minimum so it leaves the edge in a curve
//! rather than a corner. It leaves the edge its tip reaches furthest past and
//! slides along it as the tip moves. A pointer that reaches nowhere past the
//! box is tucked in and not drawn.

use crate::editor::annotations::geometry::{add, length, scale, subtract, ArrowGeometry};
use crate::editor::annotations::reveal::AnnotationReveal;

/// The padding either side of the text and above and below it, in ems.
const PAD_X: f64 = 0.5;
const PAD_Y: f64 = 0.32;
/// The narrowest a text is taken to be, in ems, so an empty box still has
/// room for its caret.
const MIN_TEXT_WIDTH: f64 = 0.3;
/// The corners' radius, in ems.
const CORNER: f32 = 0.4;

/// The corners' radius for type `em` high, before a small box caps it at its
/// own half size.
pub(super) fn text_corner(em: f64) -> f64 {
  f64::from(CORNER) * em
}

/// The pointer's base and tip radii, and how far the blend into the box
/// reaches, all in ems.
const POINTER_BASE: f32 = 0.5;
const POINTER_TIP: f32 = 0.12;
const POINTER_BLEND: f32 = 0.3;

/// The alignment a text box's native record carries in its `head`.
pub(crate) const HEAD_ALIGN_MASK: u32 = 0b11;

/// The box around a text block of `block` at `font_px`, in the pixels both
/// are in.
pub(crate) fn box_size(block: [f64; 2], font_px: f64) -> [f64; 2] {
  let font = font_px.max(0.0);
  [
    block[0].max(MIN_TEXT_WIDTH * font) + 2.0 * PAD_X * font,
    block[1] + 2.0 * PAD_Y * font,
  ]
}

/// One axis of a pointer as the native record carries it: the share of the
/// half size the tip sits at, and how many ems past the edge it reaches. The
/// twin of `TextPointer::encoded`.
fn text_pointer_axis(encoded: f32) -> (f32, f32) {
  if encoded.abs() > 1.0 {
    (encoded.signum(), encoded.abs() - 1.0)
  } else {
    (encoded, 0.0)
  }
}

/// One text box prepared for drawing and picking, read out of the slots an
/// arrow fills with its curve:
///
/// - `a` and `b` are the box's corners and `rounding` their radius;
/// - `width` is the type size the box is drawn at, which is what the text is
///   rasterised at;
/// - `start_head.c` is the text block's centre, and `start_head.a` and `.b`
///   carry where the text was rasterised, which only the compositor fills;
/// - `c` is the pointer's tip, where its grip sits whether it is drawn or
///   tucked into the box;
/// - a drawn pointer has `end_head.a` as its base, `end_head.b` the centre of
///   its rounded tip, `high` and `low` the base's and the tip's radii and
///   `end_head.c.x` how far the blend reaches. Otherwise `high` is zero;
/// - `head` is the alignment.
///
/// `origin` is the box's top-left corner in the caller's pixels, `pointer`
/// the pointer as the native record encodes it, `block` the text measured at
/// `font` and `head` the record's bits. The reveal grows the box and its type
/// about the box's centre by `scale`, and draws the pointer out of the box by
/// `high`, so the box arrives before it points and stops pointing before it
/// leaves.
pub(crate) fn prepare_text(
  origin: [f32; 2],
  pointer: [f32; 2],
  block: [f32; 2],
  font: f32,
  head: u32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let grown = reveal.scale.clamp(0.0, 1.0);
  let size = box_size([f64::from(block[0]), f64::from(block[1])], f64::from(font));
  let half = [size[0] as f32 * 0.5, size[1] as f32 * 0.5];
  let centre = add(origin, half);
  let half = scale(half, grown);
  let font = font.max(0.0) * grown;
  let min = subtract(centre, half);
  let max = add(centre, half);
  let [(along_x, reach_x), (along_y, reach_y)] = pointer.map(text_pointer_axis);
  let tip = [
    centre[0] + along_x * half[0] + reach_x.copysign(along_x) * font,
    centre[1] + along_y * half[1] + reach_y.copysign(along_y) * font,
  ];
  let mut result = ArrowGeometry {
    a: min,
    b: max,
    c: tip,
    width: font,
    rounding: (CORNER * font).min(half[0]).min(half[1]),
    head: head & HEAD_ALIGN_MASK,
    ..Default::default()
  };
  result.start_head.c = centre;
  let drawn = reveal.high.clamp(0.0, 1.0);
  if font <= 0.0 || (reach_x <= 0.0 && reach_y <= 0.0) || drawn <= 0.0 {
    return result;
  }
  let base_radius = (POINTER_BASE * font).min(half[0]).min(half[1]);
  let blend = POINTER_BLEND * font;
  // The pointer leaves the edge it reaches furthest past, level with its tip
  // along that edge. Its round base sits a base radius inside the edge it
  // leaves, and further than the blend from the edges beside it: a base
  // circle that grazed an edge would swell the blend into a bulge there.
  let leaves_side = reach_x >= reach_y;
  let (exit_axis, side_axis) = if leaves_side { (0, 1) } else { (1, 0) };
  let exit_sign = if exit_axis == 0 { along_x } else { along_y }.signum();
  let inset = base_radius + blend;
  let low = min[side_axis] + inset;
  let high = max[side_axis] - inset;
  let across = if low > high {
    centre[side_axis]
  } else {
    tip[side_axis].clamp(low, high)
  };
  let edge = centre[exit_axis] + exit_sign * half[exit_axis];
  let mut base = [0.0; 2];
  base[exit_axis] = edge - exit_sign * base_radius;
  base[side_axis] = across;
  let mut exit = base;
  exit[exit_axis] = edge;
  // Drawn out of the box from where it leaves, as far as the reveal has got.
  let tip = add(exit, scale(subtract(tip, exit), drawn));
  let reach = subtract(tip, base);
  let distance = length(reach);
  let tip_radius = POINTER_TIP * font;
  if distance <= base_radius - tip_radius {
    return result;
  }
  result.end_head.a = base;
  result.end_head.b = subtract(tip, scale(reach, tip_radius / distance));
  result.end_head.c = [blend, 0.0];
  result.high = base_radius;
  result.low = tip_radius;
  result
}

/// How far `point` falls outside a prepared text box, pointer included, in
/// the space it was prepared in. Negative inside, zero on the edge. The twin
/// of `annotation_text_distance` in the Metal and HLSL layers.
pub(crate) fn text_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let body = rounded_box_distance(point, geometry.a, geometry.b, geometry.rounding);
  if geometry.high <= 0.0 {
    return body;
  }
  let pointer = tapered_distance(
    point,
    geometry.end_head.a,
    geometry.end_head.b,
    geometry.high,
    geometry.low,
  );
  smooth_min(body, pointer, geometry.end_head.c[0])
}

/// A rounded rectangle's signed distance.
pub(crate) fn rounded_box_distance(
  point: [f32; 2],
  min: [f32; 2],
  max: [f32; 2],
  radius: f32,
) -> f32 {
  let centre = scale(add(min, max), 0.5);
  let half = scale(subtract(max, min), 0.5);
  let local = subtract(point, centre);
  let q = [
    local[0].abs() - half[0] + radius,
    local[1].abs() - half[1] + radius,
  ];
  length([q[0].max(0.0), q[1].max(0.0)]) + q[0].max(q[1]).min(0.0) - radius
}

/// The signed distance to a stroke tapering from a circle of `from_radius`
/// at `from` to one of `to_radius` at `to`, its sides tangent to both.
fn tapered_distance(
  point: [f32; 2],
  from: [f32; 2],
  to: [f32; 2],
  from_radius: f32,
  to_radius: f32,
) -> f32 {
  let axis = subtract(to, from);
  let span = axis[0] * axis[0] + axis[1] * axis[1];
  let taper = from_radius - to_radius;
  let local = subtract(point, from);
  if span <= taper * taper {
    return (length(local) - from_radius).min(length(subtract(point, to)) - to_radius);
  }
  // In the axis's own frame, scaled by its length: `q.y` runs along it and
  // `q.x` across, folded onto one side.
  let q = [
    (local[0] * axis[1] - local[1] * axis[0]).abs() / span,
    (local[0] * axis[0] + local[1] * axis[1]) / span,
  ];
  let side = [(span - taper * taper).sqrt(), taper];
  let cross = side[0] * q[1] - side[1] * q[0];
  let along = side[0] * q[0] + side[1] * q[1];
  let squared = q[0] * q[0] + q[1] * q[1];
  if cross < 0.0 {
    (span * squared).sqrt() - from_radius
  } else if cross > side[0] {
    (span * (squared + 1.0 - 2.0 * q[1])).sqrt() - to_radius
  } else {
    along - from_radius
  }
}

/// The union of two distances, rounded where they meet over `reach`.
fn smooth_min(a: f32, b: f32, reach: f32) -> f32 {
  if reach <= 0.0 {
    return a.min(b);
  }
  let blend = (reach - (a - b).abs()).max(0.0) / reach;
  a.min(b) - blend * blend * reach * 0.25
}
