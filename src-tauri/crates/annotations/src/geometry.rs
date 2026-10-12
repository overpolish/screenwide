// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The draw record every kind fills, and the vector arithmetic it is built
//! with. The twin of the typedefs in `geometry.h`, which carries no
//! arithmetic of its own.
//!
//! The compositor prepares from here, in each kind's own `geometry` module,
//! and the macOS chrome reaches the same functions through `ffi`. Everything
//! is prepared once per annotation before drawing or picking, never per
//! pixel; the shaders read the record and hold the only per-pixel copies of a
//! kind's shape.

/// Three vertices of a head's inner triangle, in the caller's pixel space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArrowTriangle {
  pub a: [f32; 2],
  pub b: [f32; 2],
  pub c: [f32; 2],
}

/// One prepared arrow, matching C's `AnnotationArrowGeometry` and the WGSL
/// storage buffer element byte for byte.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArrowGeometry {
  pub a: [f32; 2],
  pub b: [f32; 2],
  pub c: [f32; 2],
  pub width: f32,
  pub low: f32,
  pub high: f32,
  pub start_head: ArrowTriangle,
  pub end_head: ArrowTriangle,
  pub rounding: f32,
  pub head: u32,
}

const _: () = assert!(std::mem::size_of::<ArrowGeometry>() == 92);

pub(super) fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
  [a[0] + b[0], a[1] + b[1]]
}

pub(super) fn subtract(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
  [a[0] - b[0], a[1] - b[1]]
}

pub(super) fn scale(a: [f32; 2], scale: f32) -> [f32; 2] {
  [a[0] * scale, a[1] * scale]
}

pub(super) fn length(a: [f32; 2]) -> f32 {
  (a[0] * a[0] + a[1] * a[1]).sqrt()
}

pub(super) fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
  length(subtract(a, b))
}

pub(crate) fn bezier(a: [f32; 2], b: [f32; 2], c: [f32; 2], t: f32) -> [f32; 2] {
  let leg = subtract(b, a);
  let bend = add(subtract(a, scale(b, 2.0)), c);
  add(a, scale(add(scale(leg, 2.0), scale(bend, t)), t))
}
