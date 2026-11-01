// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A lens's retained draw record.
//!
//! `p0` and `p2` are its box's corners, both placed like any point, and `p1`
//! its centre, placed with them; `width` is the lens's width. `params`
//! carries its zoom, which no placement touches, being a ratio, and the
//! corner radius as a percentage of the shorter side. `flags` carries
//! `SHADOW` where the lens casts one.

use crate::flags::SHADOW;
use crate::native::NativeAnnotation;
use crate::{AnnotationPoint, AnnotationStyle};

pub(crate) fn draw_points(start: AnnotationPoint, end: AnnotationPoint) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [
      ((start.x + end.x) / 2.0) as f32,
      ((start.y + end.y) / 2.0) as f32,
    ],
    [end.x as f32, end.y as f32],
  ]
}

/// Writes what a lens's record carries beyond its points.
pub(crate) fn fill(record: &mut NativeAnnotation, zoom: f64, style: &AnnotationStyle) {
  record.params = [
    zoom as f32,
    crate::outline::native::radius(style) as f32,
    0.0,
  ];
  record.flags = if style.shadow { SHADOW } else { 0 };
}
