// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a shape, and the dress a fresh one wears.
//!
//! The box is held by two corners in source pixels, `start` the top-left and
//! `end` the bottom-right; every edit writes them back in that order. The
//! style's width is the pen, in points, so a shape keeps its weight on
//! the canvas the way an arrow does, and its radius rounds the corners as a
//! percentage of the box's shorter side, as a redaction's does.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};

/// The pen a fresh shape is drawn with, in points: an arrow's, so the
/// two read as one set of marks. The twin of `DEFAULT_ANNOTATION_WIDTH` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_SHAPE_WIDTH: f64 = 8.0;

/// The dress a fresh shape is drawn in before anything has been chosen: the
/// palette's yellow, with square corners and a clean pen.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_shape_style() -> AnnotationStyle {
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
    width: NEW_SHAPE_WIDTH,
  }
}

/// A shape from `start` to `end` with a hand-drawn stroke's `seed`, in
/// `style` or in the tool's own first dress. A fresh one in the editor starts
/// with both corners at the press, for the drag that follows to pull out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_shape(
  id: String,
  [start, end]: [AnnotationPoint; 2],
  seed: u32,
  style: Option<&AnnotationStyle>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Shape { start, end, seed },
    style: style.cloned().unwrap_or_else(default_shape_style),
  }
}

/// Whether a shape is somewhere it can be drawn.
pub(crate) fn placed(start: AnnotationPoint, end: AnnotationPoint) -> bool {
  start.x.is_finite() && start.y.is_finite() && end.x.is_finite() && end.y.is_finite()
}

/// A shape aligns by its box, its edges and its centre, as a redaction does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(crate::editor::annotations::redact::model::redact_box(
    start, end,
  ))
}
