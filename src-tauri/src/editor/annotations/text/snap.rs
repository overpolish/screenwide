// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a fresh text box lands, and what a text box offers the snap engine.

use super::geometry::box_size;
use super::metrics::text_block;
use crate::editor::annotations::snap::{SnapBox, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

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

/// A fresh text box is dropped where the press lands, so the same drag
/// carries it. What snaps is the box, exactly as when a placed box is moved.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let width = annotation.style.width;
  let AnnotationShape::Text { origin, text, .. } = &mut annotation.shape else {
    return SnapResult::default();
  };
  match snap {
    Some(request) => {
      let bounds = text_box(point, text, width, request.field.source_per_output());
      let (offset, result) = request.boxed(bounds, None);
      *origin = offset.apply(point);
      result
    }
    None => {
      *origin = point;
      SnapResult::default()
    }
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
