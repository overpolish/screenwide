// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a text box offers the snap engine.

use super::geometry::box_size;
use super::look;
use crate::snap::SnapBox;
use crate::AnnotationPoint;
use screenwide_text::TextStyle;

/// A text box's rectangle in source pixels, `origin` being its top-left
/// corner and `width` its size in output points before its look's own.
pub(crate) fn text_box(
  origin: AnnotationPoint,
  text: &str,
  look: &TextStyle,
  width: f64,
  source_per_size: f64,
) -> SnapBox {
  let size = box_size(look::block(look, text, width), look::type_size(look, width));
  SnapBox {
    x: origin.x,
    y: origin.y,
    width: size[0] * source_per_size,
    height: size[1] * source_per_size,
  }
}

/// A text box aligns by its box: its edges and its centre.
pub(crate) fn field_box(
  origin: AnnotationPoint,
  text: &str,
  look: &TextStyle,
  width: f64,
  source_per_size: f64,
) -> Option<SnapBox> {
  Some(text_box(origin, text, look, width, source_per_size))
}
