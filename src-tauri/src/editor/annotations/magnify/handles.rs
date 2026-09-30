// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A magnifier's grips, as the native chrome needs them.

use crate::editor::annotations::handles::{
  normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::outline::native::radius;
use crate::editor::annotations::redact::model::{corners, redact_box};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A magnifier reads the slots as a spotlight does for its zoom area:
/// `start` is the zoom area's top-left corner and `end` its bottom-right,
/// normalised over the source, and `start_head` the corner radius, from
/// which the native side places the eight grips of the layer selection's box
/// and its radius dot. `middle` is the loupe's centre and `end_head` its
/// longer side as a share of the source's width, from which it places the
/// loupe and the grip on its rim; `width` is the rim's pen. `loupe` is the
/// loupe's centre and longer side, in source pixels.
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  (loupe, size): (AnnotationPoint, f64),
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let (top_left, bottom_right) = corners(redact_box(start, end));
  let (start_x, start_y) = normalised_point(top_left, source);
  let (end_x, end_y) = normalised_point(bottom_right, source);
  let (middle_x, middle_y) = normalised_point(loupe, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Magnify.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x,
    middle_y,
    end_x,
    end_y,
    start_head: radius(style),
    end_head: size / f64::from(source.0.max(1)),
    width: stroke_width(style, image_width),
  }
}
