// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a focus, and the dress a fresh one wears.
//!
//! A focus is a box left sharp while everything around it softens: the
//! screen blurs away from the box's edge, more where the screen is turned in
//! depth, and the background beyond the screen blurs whole. The box is held
//! by two corners in source pixels, `start` the top-left and `end` the
//! bottom-right, and rounds its corners as a percentage of its shorter side,
//! exactly as a spotlight's does. Nothing is drawn for it: the effect is the
//! blur itself, so its own style carries only the strength of the blur.

#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::snap::SnapBox;
use crate::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};

/// The corner radius a fresh focus takes: gently rounded.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub const NEW_FOCUS_RADIUS: f64 = 12.0;

/// The strength a fresh focus takes, the middle step of the five a blurred
/// redaction offers: enough to read as a lens without smearing what is
/// around the box. The twin of `DEFAULT_BLUR_STRENGTH` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub const NEW_FOCUS_STRENGTH: f64 = 3.0;

/// The dress a fresh focus wears before anything has been chosen. It has no
/// colour of its own: the black it carries is never drawn, and the hover
/// halo reads it for a contrasting tone as a spotlight's does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn default_focus_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: "#000000".to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: NEW_FOCUS_RADIUS,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: NEW_FOCUS_STRENGTH,
    width: 1.0,
  }
}

/// A focus from `start` to `end`, in `style` or in the tool's own first
/// dress. A fresh one starts with both corners at the press, for the drag
/// that follows to pull out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn new_focus(
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
    reveal: crate::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Focus { start, end },
    style: style.cloned().unwrap_or_else(default_focus_style),
  }
}

/// Whether a focus is somewhere it can be drawn.
pub(crate) fn placed(start: AnnotationPoint, end: AnnotationPoint) -> bool {
  start.x.is_finite() && start.y.is_finite() && end.x.is_finite() && end.y.is_finite()
}

/// A focus aligns by its box, its edges and its centre, as a redaction does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(crate::redact::model::redact_box(start, end))
}

/// A style's share of the box's shorter side, held to the range a box can
/// be rounded over.
pub(crate) fn share(percent: f64) -> f64 {
  if percent.is_finite() {
    percent.clamp(0.0, 50.0)
  } else {
    0.0
  }
}
