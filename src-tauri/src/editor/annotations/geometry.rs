// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The draw record every kind fills, and the vector arithmetic it is built
//! with. The twin of the shared half of `geometry.h`.
//!
//! The Metal compositor prepares its annotations through the C header; the
//! D3D11 one prepares them in each kind's own `geometry` module, in the same
//! single-precision arithmetic and the same order, so the two backends draw
//! the same pixels from the same annotation. Everything is prepared once per
//! annotation before drawing or picking, never per pixel.

/// Three vertices of a head's inner triangle, in the caller's pixel space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ArrowTriangle {
  pub(crate) a: [f32; 2],
  pub(crate) b: [f32; 2],
  pub(crate) c: [f32; 2],
}

/// One prepared arrow, matching C's `AnnotationArrowGeometry` and the HLSL
/// structured buffer element byte for byte.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ArrowGeometry {
  pub(crate) a: [f32; 2],
  pub(crate) b: [f32; 2],
  pub(crate) c: [f32; 2],
  pub(crate) width: f32,
  pub(crate) low: f32,
  pub(crate) high: f32,
  pub(crate) start_head: ArrowTriangle,
  pub(crate) end_head: ArrowTriangle,
  pub(crate) rounding: f32,
  pub(crate) head: u32,
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
