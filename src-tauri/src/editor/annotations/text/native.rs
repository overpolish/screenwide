// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box's points in the retained draw record.

use super::metrics::text_block;
use super::model::TextPointer;
use crate::editor::annotations::{AnnotationPoint, AnnotationStyle};

/// A text box puts its top-left corner in `p0`, its pointer - encoded by
/// [`TextPointer::encoded`], against the box rather than in the picture - in
/// `p1`, and its text block, measured at the record's `width`, in `p2`. Only
/// `p0` is a point; the other two are placed by nothing. The alignment rides
/// in `head` and the text itself in the side buffer.
pub(crate) fn draw_points(
  origin: AnnotationPoint,
  pointer: &TextPointer,
  text: &str,
  style: &AnnotationStyle,
) -> [[f32; 2]; 3] {
  let block = text_block(text, style.width);
  [
    [origin.x as f32, origin.y as f32],
    pointer.encoded(),
    [block[0] as f32, block[1] as f32],
  ]
}
