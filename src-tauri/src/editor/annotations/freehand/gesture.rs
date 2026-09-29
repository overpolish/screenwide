// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing a stroke out, and moving one.
//!
//! A fresh stroke takes every sample that has moved at least a screen point
//! from the last one it kept, so a hand held still stores nothing and a fast
//! one keeps every turn it made. A placed stroke is held by its box: the box
//! is moved and resized as a shape's is, against the box the press began on,
//! and the stroke's points are carried and scaled into the box it ends up.

use super::model::{bounds, MAX_DRAW_POINTS};
use crate::editor::annotations::box_gesture::{carry, resize};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::redact::model::redact_box;
use crate::editor::annotations::snap::{SnapBox, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Carry the stroke being drawn on to `point`.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
) {
  let AnnotationShape::Draw { points, .. } = &mut annotation.shape else {
    return;
  };
  // A screen point in source pixels, or one source pixel where the zoom is
  // not known.
  let step = if origin.source_per_point > 0.0 {
    origin.source_per_point
  } else {
    1.0
  };
  extend(points, point, step);
}

/// Adds `point` to a stroke being drawn, where it is at least `step` from the
/// last point kept, so a hand held still stores nothing.
pub(crate) fn extend(points: &mut Vec<AnnotationPoint>, point: AnnotationPoint, step: f64) {
  if !point.x.is_finite() || !point.y.is_finite() || points.len() >= MAX_DRAW_POINTS {
    return;
  }
  let far_enough = points
    .last()
    .is_none_or(|last| (point.x - last.x).hypot(point.y - last.y) >= step);
  if far_enough {
    points.push(point);
  }
}

/// Move one grip of a stroke's box to `point`: an edge or corner scales the
/// stroke into the box it pulls out, with Shift holding a corner to the
/// box's proportions, and the body carries the whole stroke.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let AnnotationShape::Draw { points: from, .. } = &origin.shape else {
    return SnapResult::default();
  };
  let (low, high) = bounds(from);
  let before = redact_box(low, high);
  let (after, result) = match handle {
    AnnotationHandle::Edges(edges) => resize(before, edges, point, shift, snap),
    AnnotationHandle::Radius => return SnapResult::default(),
    _ => carry(before, point, origin.point, snap),
  };
  if let AnnotationShape::Draw { points, .. } = &mut annotation.shape {
    *points = from
      .iter()
      .map(|point| rescaled(*point, before, after))
      .collect();
  }
  result
}

/// `point` of a stroke held by the box `from`, where it lands once the box is
/// `to`. A box with no width or height has nothing to scale along that axis,
/// and is carried along it instead.
fn rescaled(point: AnnotationPoint, from: SnapBox, to: SnapBox) -> AnnotationPoint {
  let axis = |at: f64, from_start: f64, from_size: f64, to_start: f64, to_size: f64| {
    if from_size > 0.0 {
      to_start + (at - from_start) * to_size / from_size
    } else {
      to_start + (at - from_start)
    }
  };
  AnnotationPoint {
    x: axis(point.x, from.x, from.width, to.x, to.width),
    y: axis(point.y, from.y, from.height, to.y, to.height),
  }
}
