// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a text box offers the snap engine.

use super::geometry::box_size;
use super::metrics::text_block;
use crate::editor::annotations::snap::SnapBox;
use crate::editor::annotations::AnnotationPoint;

/// A text box's rectangle in source pixels, `origin` being its top-left
/// corner and `width` its type size in output pixels.
pub(crate) fn text_box(
  origin: AnnotationPoint,
  text: &str,
  width: f64,
  source_per_output: f64,
) -> SnapBox {
  let size = box_size(text_block(text, width), width);
  SnapBox {
    x: origin.x,
    y: origin.y,
    width: size[0] * source_per_output,
    height: size[1] * source_per_output,
  }
}

/// A text box aligns by its box: its edges and its centre.
pub(crate) fn field_box(
  origin: AnnotationPoint,
  text: &str,
  width: f64,
  source_per_output: f64,
) -> Option<SnapBox> {
  Some(text_box(origin, text, width, source_per_output))
}
