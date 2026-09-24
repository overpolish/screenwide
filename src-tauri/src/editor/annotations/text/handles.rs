// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box's grips, as the native chrome needs them.

use super::metrics::text_block;
use super::model::TextPointer;
use crate::editor::annotations::handles::{
  normalised_point, stroke_width, NativeAnnotationHandles,
};
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationStyle};

/// A text box reads the arrow's slots its own way: `start` is the box's
/// top-left corner, normalised over the source; `end` is its pointer as
/// [`TextPointer::encoded`] carries it, against the box and never normalised;
/// `middle` is the text block's width and height and `width` the type size,
/// all as shares of the drawn width, so the native side places the box and
/// its pointer in isotropic display points; `start_head` is the alignment.
pub(crate) fn grips(
  origin: AnnotationPoint,
  pointer: &TextPointer,
  text: &str,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let (start_x, start_y) = normalised_point(origin, source);
  let [end_x, end_y] = pointer.encoded();
  let share = stroke_width(style, image_width);
  let block = text_block(text, style.width);
  let per_output = if style.width > 0.0 {
    share / style.width
  } else {
    0.0
  };
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Text.raw(),
    padding: 0,
    start_x,
    start_y,
    middle_x: block[0] * per_output,
    middle_y: block[1] * per_output,
    end_x: f64::from(end_x),
    end_y: f64::from(end_y),
    start_head: f64::from(style.align.raw()),
    end_head: 0.0,
    width: share,
  }
}
