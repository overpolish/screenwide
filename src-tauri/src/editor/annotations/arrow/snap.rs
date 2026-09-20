// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a fresh arrow does while it is drawn out, and what an arrow offers
//! the snap engine.

use crate::editor::annotations::snap::{SnapBox, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// A fresh arrow is drawn out from where the press landed: the drag carries
/// the end grip, and the curve stays straight behind it. It snaps the way
/// that grip does on an arrow already placed.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let mut result = SnapResult::default();
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = &mut annotation.shape
  else {
    return result;
  };
  let tip = result.tip(point, snap);
  *end = tip;
  *control = AnnotationPoint {
    x: (start.x + tip.x) / 2.0,
    y: (start.y + tip.y) / 2.0,
  };
  result
}

/// An arrow contributes nothing to the field. It is a line rather than a
/// block, so aligning one by a bounding box would line up a rectangle that is
/// not drawn anywhere.
pub(crate) fn field_box() -> Option<SnapBox> {
  None
}
