// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A counter's three points in the retained draw record.

use crate::editor::annotations::AnnotationPoint;

/// A counter puts its centre in `p0`, the direction of its tail in `p1[0]` -
/// radians clockwise from east - and its number in `p1[1]`; `width` carries
/// the disc's diameter. `p2` repeats the centre so a bounding box over the
/// three points is still the annotation's.
pub(crate) fn draw_points(center: AnnotationPoint, value: u32, angle: f64) -> [[f32; 2]; 3] {
  let centre = [center.x as f32, center.y as f32];
  [centre, [angle as f32, value as f32], centre]
}
