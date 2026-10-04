// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sticker's chrome beyond the box it fits in: where its turned picture is
//! picked, and the grip above its top side that turns it. The twin of the
//! sticker's branches in `recording_preview_surface_macos+annotation_picking.m`
//! and `recording_preview_annotation_geometry_macos.h`.

use super::*;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::sticker::geometry::{prepare_sticker, sticker_distance};

/// A sticker prepared in display points, from the grips it was published
/// with: its middle in `middle`, its turn in `start_head`, and its half width
/// and half height, as shares of the drawn width, in `end_head` and `width`.
fn geometry(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> ArrowGeometry {
  let (x, y) = super::picking::display_point(image, item.middle_x, item.middle_y);
  let (sin, cos) = item.start_head.sin_cos();
  let (half_width, half_height) = (item.end_head * image.width, item.width * image.width);
  prepare_sticker(
    [x as f32, y as f32],
    [(x + cos * half_width) as f32, (y + sin * half_width) as f32],
    [
      (x - sin * half_height) as f32,
      (y + cos * half_height) as f32,
    ],
    0.0,
    false,
    AnnotationReveal::WHOLE,
  )
}

/// How far `point` is from the turned picture, in display points: zero or
/// less anywhere on it.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  sticker_distance([point.0 as f32, point.1 as f32], &geometry(image, item))
}

/// Where the turning grip sits: above the picture's top side, where the
/// preparation placed it.
pub(super) fn turn_grip(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> (f64, f64) {
  let grip = geometry(image, item).end_head.c;
  (f64::from(grip[0]), f64::from(grip[1]))
}
