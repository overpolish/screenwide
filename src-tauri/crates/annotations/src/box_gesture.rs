// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a box's grips does to it: a redaction's, a shape's or a
//! spotlight's.
//!
//! A box has the layer selection's eight grips: a corner moves two sides
//! and an edge one. Every sample is measured against the box the press began
//! on, so a side dragged past its opposite simply turns the box inside out
//! and the corners are written back top-left first.

use super::redact::model::{corners, redact_box};
use super::redact::snap::snap_sides;
use crate::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::snap::{SnapBox, SnapOffset, SnapRequest, SnapResult};
use crate::{Annotation, AnnotationPoint, AnnotationShape};

/// Which sides a box grip moves, as the native `ScreenwideAnnotationHandleBox`
/// bits carry them. The same bits the resize cursors are chosen by.
pub(crate) const EDGE_LEFT: u32 = 1;
pub(crate) const EDGE_RIGHT: u32 = 1 << 1;
pub(crate) const EDGE_TOP: u32 = 1 << 2;
pub(crate) const EDGE_BOTTOM: u32 = 1 << 3;

/// Move one grip of a box to `point`. An edge or corner grip resizes the
/// box, with Shift holding a corner to the box's own proportions; the body
/// carries the whole box.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let Some((from_start, from_end)) = held_corners(&origin.shape) else {
    return SnapResult::default();
  };
  let from = redact_box(from_start, from_end);
  let (bounds, result) = match handle {
    AnnotationHandle::Edges(edges) => resize(from, edges, point, shift, snap),
    _ => carry(from, point, origin.point, snap),
  };
  write(annotation, bounds);
  result
}

/// Pull a fresh box out from where the press landed: the press is one
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

/// The two corners a boxed shape is held by, or `None` for a shape that is
/// not a box.
fn held_corners(shape: &AnnotationShape) -> Option<(AnnotationPoint, AnnotationPoint)> {
  match shape {
    AnnotationShape::Redact { start, end, .. }
    | AnnotationShape::Shape { start, end, .. }
    | AnnotationShape::Spotlight { start, end }
    | AnnotationShape::Focus { start, end }
    | AnnotationShape::Magnify { start, end, .. } => Some((*start, *end)),
    _ => None,
  }
}

fn write(annotation: &mut Annotation, bounds: SnapBox) {
  if let AnnotationShape::Redact { start, end, .. }
  | AnnotationShape::Shape { start, end, .. }
  | AnnotationShape::Spotlight { start, end }
  | AnnotationShape::Focus { start, end }
  | AnnotationShape::Magnify { start, end, .. } = &mut annotation.shape
  {
    (*start, *end) = corners(bounds);
  }
}

pub(crate) fn carry(
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

pub(crate) fn resize(
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
mod tests;
