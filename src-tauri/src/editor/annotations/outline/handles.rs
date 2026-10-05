// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shape's grips, as the native chrome needs them.

use super::native::{hand, radius};
use crate::editor::annotations::handles::{
  normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::redact::model::{corners, redact_box};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A shape reads the slots as a redaction does: `start` is the box's
/// top-left corner and `end` its bottom-right, normalised over the source,
/// `middle` its centre and `start_head` the corner radius, from which the
/// native side places the eight grips of the layer selection's box and its
/// radius dot. `end_head` carries the stroke's hand and `width` its pen, as a
/// share of the drawn width, so the chrome picks the stroke as it is drawn.
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  seed: u32,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let (top_left, bottom_right) = corners(redact_box(start, end));
  let (start_x, start_y) = normalised_point(top_left, source);
  let (end_x, end_y) = normalised_point(bottom_right, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Shape.raw(),
    flags: 0,
    start_x,
    start_y,
    middle_x: (start_x + end_x) / 2.0,
    middle_y: (start_y + end_y) / 2.0,
    end_x,
    end_y,
    start_head: radius(style),
    end_head: f64::from(hand(style.hand_drawn, seed)),
    width: stroke_width(style, image_width),
    radius: 0.0,
  }
}
