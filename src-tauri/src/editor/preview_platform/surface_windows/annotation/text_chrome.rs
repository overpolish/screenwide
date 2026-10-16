// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box's chrome: the box and tail its grip and its picking read, in
//! display points. The twin of the text box's branch in
//! `recording_preview_annotation_geometry_macos.h`.

use super::picking::display_point;
use super::*;
use screenwide_annotations::geometry::ArrowGeometry;
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::geometry::{prepare_text, HEAD_TAIL_PLACED};

/// A text box prepared in display points, from the grips it was published
/// with. The twin of the text branch of `annotation_prepared`.
pub(super) fn text_geometry(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> ArrowGeometry {
  let origin = display_point(image, item.start_x, item.start_y);
  let head = item.start_head as u32;
  // The tail's tip is held against the box, never placed, unless it was
  // put on the picture.
  let tip = if head & HEAD_TAIL_PLACED != 0 {
    let (x, y) = display_point(image, item.end_x, item.end_y);
    [x as f32, y as f32]
  } else {
    [item.end_x as f32, item.end_y as f32]
  };
  prepare_text(
    [origin.0 as f32, origin.1 as f32],
    tip,
    [
      (item.middle_x * image.width) as f32,
      (item.middle_y * image.width) as f32,
    ],
    (item.width * image.width) as f32,
    head,
    screenwide_text::TailShape {
      base: item.radius as f32,
      tip: item.tilt_x as f32,
      blend: item.tilt_y as f32,
    },
    AnnotationReveal::WHOLE,
  )
}
