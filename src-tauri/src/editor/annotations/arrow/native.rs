// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An arrow's three points in the retained draw record.

use crate::editor::annotations::AnnotationPoint;

/// An arrow fills `p0`, `p1` and `p2` with its Bézier's start, control and
/// end; `width` carries its stroke.
pub(crate) fn draw_points(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [control.x as f32, control.y as f32],
    [end.x as f32, end.y as f32],
  ]
}
