// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's chrome: where its turned picture is picked, its own frame -
//! turned with the picture, whose eight grips move the picture's own sides -
//! its radius dot, and the grip above its top side that turns it. The frame
//! is `image::frame`'s. The twin of
//! `recording_preview_surface_macos+annotation_image.m` and the image's
//! branches in `recording_preview_surface_macos+annotation_picking.m` and
//! `recording_preview_annotation_geometry_macos.h`.

use super::*;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::gesture::{BOX_HANDLES, RADIUS_HANDLE};
use crate::editor::annotations::image::frame::{cursor_sides, ImageFrame};
use crate::editor::annotations::image::geometry::{image_distance, prepare_image};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::AnnotationPoint;

/// An image prepared in display points, from the grips it was published
/// with: its middle in `middle`, its turn in `start_head`, and its half width
/// and half height, as shares of the drawn width, in `end_head` and `width`.
fn geometry(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> ArrowGeometry {
  let (x, y) = super::picking::display_point(image, item.middle_x, item.middle_y);
  let (sin, cos) = item.start_head.sin_cos();
  let (half_width, half_height) = (item.end_head * image.width, item.width * image.width);
  prepare_image(
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

/// The picture's own frame in display points, from the same slots.
fn own_frame(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> ImageFrame {
  let (x, y) = super::picking::display_point(image, item.middle_x, item.middle_y);
  ImageFrame {
    center: AnnotationPoint { x, y },
    angle: item.start_head,
    half_width: item.end_head * image.width,
    half_height: item.width * image.width,
  }
}

/// How far `point` is from the turned picture, in display points: zero or
/// less anywhere on it.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  image_distance([point.0 as f32, point.1 as f32], &geometry(image, item))
}

/// Where the turning grip sits: above the picture's top side, where the
/// preparation placed it.
pub(super) fn turn_grip(image: PreviewSurfaceRect, item: &NativeAnnotationHandles) -> (f64, f64) {
  let grip = geometry(image, item).end_head.c;
  (f64::from(grip[0]), f64::from(grip[1]))
}

/// The grip under `point`, as the handle it reports: the frame's eight,
/// then its radius dot, then the turning grip. The frame's come first, so on
/// a picture too small to keep them apart a corner wins over the dot.
pub(super) fn grip_at(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  let near =
    |(x, y): (f64, f64)| (point.0 - x).abs() <= HANDLE_HIT && (point.1 - y).abs() <= HANDLE_HIT;
  own_frame(image, item)
    .grips(item.radius)
    .into_iter()
    .find(|(grip, _)| near((grip.x, grip.y)))
    .map(|(_, handle)| handle)
    .or_else(|| near(turn_grip(image, item)).then_some(HANDLE_TAIL))
}

/// The resize cursor a frame grip or the radius dot shows, turned with the
/// picture; `None` for any other grip. The radius dot drags along the
/// picture's own diagonal, as a box's dot drags along its upright one.
pub(super) fn grip_cursor(item: &NativeAnnotationHandles, handle: u32) -> Option<u32> {
  let sides = if handle == RADIUS_HANDLE {
    1 | 4
  } else {
    handle
      .checked_sub(BOX_HANDLES)
      .filter(|sides| *sides < 16)?
  };
  Some(BOX_HANDLES + cursor_sides(sides, item.start_head))
}

/// The chosen image's frame in device pixels: its four corners, clockwise
/// from the picture's own top-left, then its radius dot. `None` when no image
/// is chosen or the annotation tool has no say.
pub(crate) fn selected_frame(state: &SurfaceState, scale: f64) -> Option<[[f32; 2]; 5]> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  let item = selected_item(state)?;
  if item.shape_kind() != AnnotationKind::Image {
    return None;
  }
  let frame = own_frame(image_frame(state)?, item);
  let device = |point: AnnotationPoint| [(point.x * scale) as f32, (point.y * scale) as f32];
  let [a, b, c, d] = frame.corners().map(device);
  Some([a, b, c, d, device(frame.radius_dot(item.radius))])
}
