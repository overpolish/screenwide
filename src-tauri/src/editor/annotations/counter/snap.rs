// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a fresh counter lands, and what a counter offers the snap engine.

use super::silhouette::counter_tail_tip;
use crate::editor::annotations::snap::{disc_radius, SnapBox, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// A fresh counter is dropped whole where the press lands, so the same drag
/// carries it. What snaps is the disc, not the pointer, exactly as it is when
/// an already placed counter is moved.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let width = annotation.style.width;
  let AnnotationShape::Counter { center, angle, .. } = &mut annotation.shape else {
    return SnapResult::default();
  };
  match snap {
    Some(request) => {
      let radius = request.field.radius(width);
      let (offset, result) = request.boxed(
        request.field.disc(point, width),
        Some(counter_tail_tip(point, radius, *angle)),
      );
      *center = offset.apply(point);
      result
    }
    None => {
      *center = point;
      SnapResult::default()
    }
  }
}

/// A counter's alignment rectangle is its disc, in the source's pixels: the
/// one annotation shape with a block to line up by.
pub(crate) fn field_box(
  center: AnnotationPoint,
  width: f64,
  source_per_output: f64,
) -> Option<SnapBox> {
  Some(SnapBox::disc(center, disc_radius(width, source_per_output)))
}
