// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making an arrow, and the dress a fresh one wears.
//!
//! An arrow is one quadratic Bézier and a stroke. What it may be bent to
//! lives in [`super::bend`], which every edit runs the shape back through.

use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

/// The stroke a fresh arrow is drawn with, in output pixels.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_ARROW_WIDTH: f64 = 8.0;

/// The dress a fresh arrow is drawn in before anything has been chosen.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_arrow_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::End,
    width: NEW_ARROW_WIDTH,
  }
}

/// A straight arrow in `style`, or in the tool's own first dress where the
/// editor has not settled on one yet.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_arrow(
  id: String,
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: Option<&AnnotationStyle>,
) -> Annotation {
  let mut arrow = Annotation {
    above_camera: false,
    animated: true,
    id,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Arrow {
      start,
      control: AnnotationPoint {
        x: (start.x + end.x) / 2.0,
        y: (start.y + end.y) / 2.0,
      },
      end,
    },
    style: style.cloned().unwrap_or_else(default_arrow_style),
  };
  super::bend::clamp_bend(&mut arrow);
  arrow
}

/// Whether an arrow is somewhere it can be drawn. A document read from disk
/// carries whatever it was written with, so every space-changing path tests
/// this before it works with one.
pub(crate) fn placed(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
) -> bool {
  [start, control, end]
    .iter()
    .all(|point| point.x.is_finite() && point.y.is_finite())
}
