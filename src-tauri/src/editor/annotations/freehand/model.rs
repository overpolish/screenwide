// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a stroke, and the dress a fresh one wears.
//!
//! The style's width is the pen, in points, so a stroke keeps its weight on
//! the canvas the way an arrow does.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};

/// The pen a fresh stroke is drawn with, in points: an arrow's, so the two
/// read as one set of marks. The twin of `DEFAULT_ANNOTATION_WIDTH` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_DRAW_WIDTH: f64 = 8.0;

/// The most points one stroke keeps. A stroke is thinned as it is drawn, so
/// this is several screens of slow scribbling; past it a stroke is not one a
/// document can be trusted to hold.
pub(crate) const MAX_DRAW_POINTS: usize = 4096;

/// The dress a fresh stroke is drawn in before anything has been chosen: the
/// palette's yellow at an arrow's pen.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_draw_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    radius: 0.0,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: 0.0,
    width: NEW_DRAW_WIDTH,
  }
}

/// A stroke begun at `point`, in `style` or in the tool's own first dress.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_draw(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: true,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Draw {
      points: vec![point],
      smooth: false,
    },
    style: style.cloned().unwrap_or_else(default_draw_style),
  }
}

/// Whether a stroke is somewhere it can be drawn: at least one point, none
/// past the most a stroke keeps, all of them on the picture's plane.
pub(crate) fn placed(points: &[AnnotationPoint]) -> bool {
  !points.is_empty()
    && points.len() <= MAX_DRAW_POINTS
    && points
      .iter()
      .all(|point| point.x.is_finite() && point.y.is_finite())
}

/// The box round a stroke's points: its top-left and bottom-right corners.
/// A stroke that is not placed has none, and reads as the origin.
pub(crate) fn bounds(points: &[AnnotationPoint]) -> (AnnotationPoint, AnnotationPoint) {
  let Some(first) = points.first() else {
    return (AnnotationPoint::default(), AnnotationPoint::default());
  };
  points.iter().fold((*first, *first), |(low, high), point| {
    (
      AnnotationPoint {
        x: low.x.min(point.x),
        y: low.y.min(point.y),
      },
      AnnotationPoint {
        x: high.x.max(point.x),
        y: high.y.max(point.y),
      },
    )
  })
}

/// A stroke aligns by its box, its edges and its centre, as a shape does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(points: &[AnnotationPoint]) -> Option<SnapBox> {
  let (low, high) = bounds(points);
  Some(crate::editor::annotations::redact::model::redact_box(
    low, high,
  ))
}
