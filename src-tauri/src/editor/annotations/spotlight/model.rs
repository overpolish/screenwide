// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a spotlight, and the dress a fresh one wears.
//!
//! The box is held by two corners in source pixels, `start` the top-left and
//! `end` the bottom-right; every edit writes them back in that order. Its
//! radius rounds the corners and its softness fades the edge, both as a
//! percentage of the box's shorter side, so a spotlight looks the same at
//! every size the picture is drawn at.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};

/// How much of the light the shade outside a spotlight takes away. Fixed
/// rather than a choice: dark enough that the eye goes to the light, light
/// enough that what is around it can still be read. The twin of
/// `annotation_spotlight_dim` in the Metal and HLSL spotlight passes.
#[cfg(all(test, target_os = "macos"))]
pub(crate) const SPOTLIGHT_DIM: f32 = 0.4;

/// The softness a fresh spotlight's edge fades over, as a percentage of its
/// shorter side: enough to read as light rather than a cut-out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_SPOTLIGHT_SOFTNESS: f64 = 10.0;

/// The corner radius a fresh spotlight takes: gently rounded, as light is.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_SPOTLIGHT_RADIUS: f64 = 12.0;

/// The dress a fresh spotlight wears before anything has been chosen. It has
/// no colour of its own: the black it carries is the shade's, which the
/// hover halo reads to find a contrasting tone. It draws no stroke either,
/// and carries a width of one only because every clip must have a usable one.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_spotlight_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: "#000000".to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: NEW_SPOTLIGHT_RADIUS,
    redaction: Default::default(),
    shadow: false,
    softness: NEW_SPOTLIGHT_SOFTNESS,
    strength: 0.0,
    width: 1.0,
  }
}

/// A spotlight from `start` to `end`, in `style` or in the tool's own first
/// dress. A fresh one in the editor starts with both corners at the press,
/// for the drag that follows to pull out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_spotlight(
  id: String,
  [start, end]: [AnnotationPoint; 2],
  style: Option<&AnnotationStyle>,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Spotlight { start, end },
    style: style.cloned().unwrap_or_else(default_spotlight_style),
  }
}

/// Whether a spotlight is somewhere it can be drawn.
pub(crate) fn placed(start: AnnotationPoint, end: AnnotationPoint) -> bool {
  start.x.is_finite() && start.y.is_finite() && end.x.is_finite() && end.y.is_finite()
}

/// A spotlight aligns by its box, its edges and its centre, as a redaction
/// does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(crate::editor::annotations::redact::model::redact_box(
    start, end,
  ))
}

/// A style's share of the box's shorter side, held to the range a box can
/// be rounded or faded over.
pub(crate) fn share(percent: f64) -> f64 {
  if percent.is_finite() {
    percent.clamp(0.0, 50.0)
  } else {
    0.0
  }
}
