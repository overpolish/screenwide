// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a text box offers the snap engine.

use super::geometry::box_size;
use super::look;
use crate::snap::SnapBox;
use crate::{AnnotationPoint, AnnotationShape};
use screenwide_text::{TextSpan, TextStyle};

/// A text box's rectangle in source pixels, `origin` being its top-left
/// corner, `emphasis` the words it sets apart and `width` its size in
/// output points before its look's own.
pub(crate) fn text_box(
  origin: AnnotationPoint,
  text: &str,
  look: &TextStyle,
  emphasis: &[TextSpan],
  width: f64,
  source_per_size: f64,
) -> SnapBox {
  let size = box_size(
    look::block(look, text, emphasis, width),
    look::type_size(look, width),
  );
  SnapBox {
    x: origin.x,
    y: origin.y,
    width: size[0] * source_per_size,
    height: size[1] * source_per_size,
  }
}

/// A text box aligns by its box: its edges and its centre. None for any
/// other shape.
pub(crate) fn field_box(
  shape: &AnnotationShape,
  width: f64,
  source_per_size: f64,
) -> Option<SnapBox> {
  let AnnotationShape::Text {
    origin,
    text,
    look,
    emphasis,
    ..
  } = shape
  else {
    return None;
  };
  Some(text_box(
    *origin,
    text,
    look,
    emphasis,
    width,
    source_per_size,
  ))
}
