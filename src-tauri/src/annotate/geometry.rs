// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving an annotation from the desktop it was drawn on into pixels.
//!
//! The overlay draws in global logical points, which is also the space the
//! cursor sidecar reports in. A recording wants the annotation in its source
//! pixels, and the overlay itself wants it in one display's layer pixels;
//! both are the same kind of offset-and-scale applied to the geometry.

use crate::editor::annotations::{Annotation, AnnotationPoint};
use crate::recording::cursor::CursorSource;

/// Whether the bounding box of `points` reaches the recorded rectangle.
fn overlaps(points: &[AnnotationPoint; 3], source: &CursorSource) -> bool {
  let horizontal = points.iter().any(|point| point.x >= source.x)
    && points
      .iter()
      .any(|point| point.x <= source.x + source.width);
  let vertical = points.iter().any(|point| point.y >= source.y)
    && points
      .iter()
      .any(|point| point.y <= source.y + source.height);
  horizontal && vertical
}

/// An annotation in the recording's source pixels, or nothing when the annotation is not
/// over the recorded pixels at all: an annotation on a display the recording does not
/// cover is drawn, but never recorded.
pub(super) fn source_annotation(
  annotation: &Annotation,
  source: &CursorSource,
) -> Option<Annotation> {
  if !(source.width > 0.0 && source.height > 0.0) {
    return None;
  }
  let points = annotation.shape.points();
  if points
    .iter()
    .any(|point| !point.x.is_finite() || !point.y.is_finite())
  {
    return None;
  }
  // A quadratic Bézier stays inside the hull of its control points, so a
  // bounding box that misses the source means the curve misses it too.
  if !overlaps(&points, source) {
    return None;
  }
  let horizontal = f64::from(source.video_width) / source.width;
  let vertical = f64::from(source.video_height) / source.height;
  // A stroke width is pixels of whatever it is drawn on, the way an editor
  // annotation's is: the same preset is the same weight live and in the
  // editor, so the number carries over rather than being rescaled.
  let width = annotation.style.width;
  if !(width.is_finite() && width > 0.0) {
    return None;
  }
  Some(Annotation {
    shape: annotation.shape.mapped(|point| AnnotationPoint {
      x: (point.x - source.x) * horizontal,
      y: (point.y - source.y) * vertical,
    }),
    style: annotation.style.clone(),
    ..annotation.clone()
  })
}

/// An annotation in one display's layer pixels: its origin is the display's top-left
/// corner in desktop points, and `scale` its backing scale. Only the geometry
/// scales - the stroke width is already in pixels, as the editor's is, so a
/// preset drawn live has the weight the same preset has on a picture.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(super) fn display_annotation(
  annotation: &Annotation,
  origin: (f64, f64),
  scale: f64,
) -> Annotation {
  Annotation {
    shape: annotation.shape.mapped(|point| AnnotationPoint {
      x: (point.x - origin.0) * scale,
      y: (point.y - origin.1) * scale,
    }),
    style: annotation.style.clone(),
    ..annotation.clone()
  }
}
