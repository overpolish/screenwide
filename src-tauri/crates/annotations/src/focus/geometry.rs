// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A focus's prepared record and the distance that picks it.
//!
//! The record carries the box's corners in `a` and `b` and its corner
//! radius in `rounding`, in the pixels it was prepared in, exactly as a
//! spotlight's does but with no soft edge: the blur falls away outside the
//! box by its own distance rule, never from a fade inside it.

use super::model::share;
use crate::geometry::ArrowGeometry;

/// The box between the two corners, in the space they are given in, rounded
/// by `radius` percent of its shorter side.
pub fn prepare_focus(start: [f32; 2], end: [f32; 2], radius: f32) -> ArrowGeometry {
  let a = [start[0].min(end[0]), start[1].min(end[1])];
  let b = [start[0].max(end[0]), start[1].max(end[1])];
  let shortest = (b[0] - a[0]).min(b[1] - a[1]).max(0.0);
  ArrowGeometry {
    a,
    b,
    rounding: shortest * share(f64::from(radius)) as f32 / 100.0,
    width: 0.0,
    ..ArrowGeometry::default()
  }
}

/// How far `point` falls from the prepared box: zero or less anywhere inside
/// it, so a press anywhere in the box picks it.
#[cfg(target_os = "macos")]
pub(crate) fn focus_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  crate::text::geometry::rounded_box_distance(point, geometry.a, geometry.b, geometry.rounding)
}
