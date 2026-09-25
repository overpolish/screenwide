// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a redaction's grips does to it.
//!
//! The box has the layer selection's eight grips: a corner moves two sides
//! and an edge one. Every sample is measured against the box the press began
//! on, so a side dragged past its opposite simply turns the box inside out
//! and the corners are written back top-left first.

use super::model::{corners, redact_box};
use super::snap::snap_sides;
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapBox, SnapOffset, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Which sides a box grip moves, as the native `ScreenwideAnnotationHandleBox`
/// bits carry them. The same bits the resize cursors are chosen by.
pub(crate) const EDGE_LEFT: u32 = 1;
pub(crate) const EDGE_RIGHT: u32 = 1 << 1;
pub(crate) const EDGE_TOP: u32 = 1 << 2;
pub(crate) const EDGE_BOTTOM: u32 = 1 << 3;

/// Move one grip of a redaction to `point`. An edge or corner grip resizes
/// the box, with Shift holding a corner to the box's own proportions; the
/// radius dot rounds its corners; the body carries the whole box.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let AnnotationShape::Redact {
    start: from_start,
    end: from_end,
    ..
  } = &origin.shape
  else {
    return SnapResult::default();
  };
  let from = redact_box(*from_start, *from_end);
  let (bounds, result) = match handle {
    AnnotationHandle::Edges(edges) => resize(from, edges, point, shift, snap),
    AnnotationHandle::Radius => {
      annotation.style.radius = radius_at(from, point, origin.source_per_point);
      return SnapResult::default();
    }
    _ => carry(from, point, origin.point, snap),
  };
  write(annotation, bounds);
  result
}

/// The screen points the radius dot sits in from the box's corner at no
/// radius, and how far further in it moves per point of radius: the layer
/// selection's own placing, which the native chrome draws the dot by.
const RADIUS_INSET_POINTS: f64 = 10.0;
const RADIUS_TRAVEL: f64 = 0.55;

/// The corner radius, as a percentage of the box's shorter side, that puts
/// the dot under `point`, read along the diagonal the way the layer
/// selection reads its own. `source_per_point` turns the dot's inset in
/// screen points into source pixels; zero where it is unknown.
pub(crate) fn radius_at(bounds: SnapBox, point: AnnotationPoint, source_per_point: f64) -> f64 {
  let shortest = bounds.width.min(bounds.height);
  if shortest.is_nan() || shortest <= 0.0 {
    return 0.0;
  }
  let along = ((point.x - bounds.x) + (point.y - bounds.y)) / 2.0;
  let inset = RADIUS_INSET_POINTS * source_per_point.max(0.0);
  let radius = (along - inset) / RADIUS_TRAVEL;
  (radius * 100.0 / shortest).clamp(0.0, 50.0)
}

/// Pull a fresh redaction out from where the press landed: the press is one
/// corner and the hand the other, and Shift makes it square.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let pressed = SnapBox {
    x: origin.point.x,
    y: origin.point.y,
    width: 0.0,
    height: 0.0,
  };
  let (bounds, result) = resize(pressed, EDGE_RIGHT | EDGE_BOTTOM, point, shift, snap);
  write(annotation, bounds);
  result
}

fn write(annotation: &mut Annotation, bounds: SnapBox) {
  if let AnnotationShape::Redact { start, end, .. } = &mut annotation.shape {
    (*start, *end) = corners(bounds);
  }
}

fn carry(
  from: SnapBox,
  point: AnnotationPoint,
  pressed: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> (SnapBox, SnapResult) {
  let moved = from.moved(SnapOffset {
    x: point.x - pressed.x,
    y: point.y - pressed.y,
  });
  match snap {
    Some(request) => {
      let (offset, result) = request.boxed(moved, None);
      (moved.moved(offset), result)
    }
    None => (moved, SnapResult::default()),
  }
}

fn resize(
  from: SnapBox,
  edges: u32,
  point: AnnotationPoint,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> (SnapBox, SnapResult) {
  let moves_x = edges & (EDGE_LEFT | EDGE_RIGHT) != 0;
  let moves_y = edges & (EDGE_TOP | EDGE_BOTTOM) != 0;
  let (point, result) = match snap {
    Some(request) => snap_sides(request, point, moves_x, moves_y),
    None => (point, SnapResult::default()),
  };
  let mut left = from.x;
  let mut top = from.y;
  let mut right = from.x + from.width;
  let mut bottom = from.y + from.height;
  if edges & EDGE_LEFT != 0 {
    left = point.x;
  }
  if edges & EDGE_RIGHT != 0 {
    right = point.x;
  }
  if edges & EDGE_TOP != 0 {
    top = point.y;
  }
  if edges & EDGE_BOTTOM != 0 {
    bottom = point.y;
  }
  if shift && moves_x && moves_y {
    // The corner opposite the grip stays put and the box keeps the shape it
    // began with: a fresh box, which has none yet, comes out square.
    let fixed_x = if edges & EDGE_LEFT != 0 { right } else { left };
    let fixed_y = if edges & EDGE_TOP != 0 { bottom } else { top };
    let (moved_x, moved_y) = (point.x - fixed_x, point.y - fixed_y);
    let ratio = if from.width > 0.0 && from.height > 0.0 {
      from.width / from.height
    } else {
      1.0
    };
    let width = moved_x.abs().max(moved_y.abs() * ratio);
    let height = width / ratio;
    let far_x = fixed_x + width.copysign(moved_x);
    let far_y = fixed_y + height.copysign(moved_y);
    (left, right) = (fixed_x, far_x);
    (top, bottom) = (fixed_y, far_y);
  }
  let bounds = redact_box(
    AnnotationPoint { x: left, y: top },
    AnnotationPoint {
      x: right,
      y: bottom,
    },
  );
  (bounds, result)
}

#[cfg(test)]
#[path = "gesture_tests.rs"]
mod tests;
