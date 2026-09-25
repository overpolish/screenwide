// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A redaction's prepared record and the distance that picks it.
//!
//! The record carries the box's corners in `a` and `b` and its corner radius
//! in `rounding`, and nothing else. Its width stays zero, which is what both
//! compositors' annotation passes skip on: a redaction has already been
//! applied to the source by the time they run, and painting it again over
//! the canvas would only blur its hard edge.

use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::text::geometry::rounded_box_distance;

/// The box between the two corners, in the space they are given in, with
/// its corners rounded by `radius` percent of its shorter side.
pub(crate) fn prepare_redact(start: [f32; 2], end: [f32; 2], radius: f32) -> ArrowGeometry {
  let a = [start[0].min(end[0]), start[1].min(end[1])];
  let b = [start[0].max(end[0]), start[1].max(end[1])];
  let shortest = (b[0] - a[0]).min(b[1] - a[1]).max(0.0);
  let share = if radius.is_finite() {
    radius.clamp(0.0, 50.0) / 100.0
  } else {
    0.0
  };
  ArrowGeometry {
    a,
    b,
    rounding: shortest * share,
    ..ArrowGeometry::default()
  }
}

/// How far `point` falls from the prepared box: zero or less anywhere inside
/// it, so a press anywhere on a redaction picks it.
pub(crate) fn redact_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  rounded_box_distance(point, geometry.a, geometry.b, geometry.rounding)
}
