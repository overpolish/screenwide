// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A magnifier's retained draw record.
//!
//! `p0` and `p2` are the zoom area's corners and `p1` the loupe's centre, all
//! placed like any point, and `width` the rim's pen. `params` carries the
//! loupe's longer side in source pixels, which the compositor scales as it
//! places the points, and the corner radius as a percentage of the shorter
//! side, which no placement touches. `flags` carries `SHADOW` where the loupe
//! casts one.

use crate::editor::annotations::flags::SHADOW;
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::{AnnotationPoint, AnnotationStyle};

pub(crate) fn draw_points(
  start: AnnotationPoint,
  end: AnnotationPoint,
  loupe: AnnotationPoint,
) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [loupe.x as f32, loupe.y as f32],
    [end.x as f32, end.y as f32],
  ]
}

/// Writes what a magnifier's record carries beyond its points.
pub(crate) fn fill(record: &mut NativeAnnotation, size: f64, style: &AnnotationStyle) {
  record.params = [
    size as f32,
    crate::editor::annotations::outline::native::radius(style) as f32,
    0.0,
  ];
  record.flags = if style.shadow { SHADOW } else { 0 };
}
