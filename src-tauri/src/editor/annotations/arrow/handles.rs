// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An arrow's grips, as the native chrome needs them.

use super::bend::curve_midpoint;
use crate::editor::annotations::handles::{
  head_reach, normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::{
  AnnotationHead, AnnotationKind, AnnotationPoint, AnnotationStyle,
};

/// An arrow fills every slot: its two tips, the point of the curve at
/// `t = 0.5` where the middle handle sits, how far each head reaches back
/// from its tip, and the stroke's own width.
pub(crate) fn grips(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let head = style.head;
  let (start_x, start_y) = normalised_point(start, source);
  let (middle_x, middle_y) = normalised_point(curve_midpoint(start, control, end), source);
  let (end_x, end_y) = normalised_point(end, source);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Arrow.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x,
    middle_y,
    end_x,
    end_y,
    start_head: head_reach(style, head == AnnotationHead::Both, image_width),
    end_head: head_reach(style, head != AnnotationHead::None, image_width),
    width: stroke_width(style, image_width),
  }
}
