// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A counter's three points in the retained draw record.

use crate::editor::annotations::AnnotationPoint;

/// A counter puts its centre in `p0` and the direction of its tail in
/// `p1[0]` - radians clockwise from east; `width` carries the disc's
/// diameter and its number travels as text in the side buffer. `p2` repeats
/// the centre so a bounding box over the three points is still the
/// annotation's.
pub(crate) fn draw_points(center: AnnotationPoint, angle: f64) -> [[f32; 2]; 3] {
  let centre = [center.x as f32, center.y as f32];
  [centre, [angle as f32, 0.0], centre]
}
