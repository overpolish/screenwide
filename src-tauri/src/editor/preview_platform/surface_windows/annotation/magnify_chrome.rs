// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A magnifier's chrome beyond its zoom area's box: where it is picked, which
//! part a press lands on, and the grip on its loupe's rim that sizes the
//! loupe. The twin of the magnifier's branches in
//! `recording_preview_surface_macos+annotation_picking.m` and
//! `recording_preview_annotation_geometry_macos.h`.

use super::*;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::magnify::geometry::{
  magnify_distance, magnify_part, prepare_magnify, MagnifyPart,
};
use crate::editor::annotations::reveal::AnnotationReveal;

/// A magnifier prepared in display points, from the grips it was published
/// with: its zoom area's box in `start` and `end`, its loupe's centre in
/// `middle`, its radius in `start_head` and its loupe's size, as a share of
/// the drawn width, in `end_head`.
fn geometry(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> ArrowGeometry {
  let place = |x: f64, y: f64| {
    let (x, y) = super::picking::display_point(image, x, y);
    [x as f32, y as f32]
  };
  prepare_magnify(
    place(item.start_x, item.start_y),
    place(item.middle_x, item.middle_y),
    place(item.end_x, item.end_y),
    (item.end_head * image.width) as f32,
    item.start_head as f32,
    (item.width * image.width) as f32,
    AnnotationReveal::WHOLE,
  )
}

/// How far `point` is from the loupe or the zoom area, in display points:
/// zero or less anywhere inside either.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  magnify_distance([point.0 as f32, point.1 as f32], &geometry(image, item))
}

/// Whether `point` lands on the loupe, which is carried on its own, rather
/// than on the zoom area.
pub(super) fn on_loupe(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> bool {
  magnify_part([point.0 as f32, point.1 as f32], &geometry(image, item)) == Some(MagnifyPart::Loupe)
}

/// Where the loupe's grip sits: on its rim towards its bottom-right corner,
/// where the preparation placed it.
pub(super) fn loupe_grip(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> (f64, f64) {
  let grip = geometry(image, item).end_head.c;
  (f64::from(grip[0]), f64::from(grip[1]))
}
