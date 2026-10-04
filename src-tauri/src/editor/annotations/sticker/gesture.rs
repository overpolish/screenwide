// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a sticker's grips does to it.
//!
//! Its grips are those of the upright box it fits in. A corner or an edge
//! sizes the whole picture, keeping its proportions, with the box's opposite
//! side held where it was; the turning grip stands the picture's top towards
//! the hand; the body carries it. Every sample is measured against the
//! sticker the press began on.

use super::model::{bounds, field_box, sticker_turn, MIN_STICKER_SIZE};
use crate::editor::annotations::box_gesture::{
  carry, EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP,
};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapBox, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// The sticker a gesture began on: its middle, longer side, turn and
/// proportions.
fn held(shape: &AnnotationShape) -> Option<(AnnotationPoint, f64, f64, f64)> {
  match shape {
    AnnotationShape::Sticker {
      center,
      size,
      angle,
      aspect,
      ..
    } => Some((*center, *size, *angle, *aspect)),
    _ => None,
  }
}

/// Move one grip of a sticker to `point`. `shift` holds its turn to the
/// eighth turns.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let Some((from_center, from_size, from_angle, aspect)) = held(&origin.shape) else {
    return SnapResult::default();
  };
  let AnnotationShape::Sticker {
    center,
    size,
    angle,
    ..
  } = &mut annotation.shape
  else {
    return SnapResult::default();
  };
  match handle {
    AnnotationHandle::Tail => {
      *angle = sticker_turn(from_center, point, from_angle, shift);
      SnapResult::default()
    }
    AnnotationHandle::Edges(edges) => {
      let (low, high) = bounds(from_center, from_size, from_angle, aspect);
      (*center, *size) = resized(low, high, from_size, edges, point);
      SnapResult::default()
    }
    _ => {
      let from = field_box(from_center, from_size, from_angle, aspect).unwrap_or(SnapBox {
        x: from_center.x,
        y: from_center.y,
        width: 0.0,
        height: 0.0,
      });
      let (moved, result) = carry(from, point, origin.point, snap);
      *center = middle(moved);
      result
    }
  }
}

/// A fresh sticker is dropped whole where the press lands, and the same drag
/// carries it; what snaps is the box it fits in, not the pointer.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let AnnotationShape::Sticker {
    center,
    size,
    angle,
    aspect,
    ..
  } = &mut annotation.shape
  else {
    return SnapResult::default();
  };
  let Some(request) = snap else {
    *center = point;
    return SnapResult::default();
  };
  let Some(dropped) = field_box(point, *size, *angle, *aspect) else {
    *center = point;
    return SnapResult::default();
  };
  let (offset, result) = request.boxed(dropped, None);
  *center = middle(dropped.moved(offset));
  result
}

fn middle(bounds: SnapBox) -> AnnotationPoint {
  AnnotationPoint {
    x: bounds.x + bounds.width * 0.5,
    y: bounds.y + bounds.height * 0.5,
  }
}

/// The middle and longer side that put the box's moved sides under `point`
/// with its opposite sides held. A corner takes whichever of its two sides
/// asks for the larger picture, so the box always reaches the hand; a side
/// dragged past its opposite stops at the smallest sticker rather than
/// turning it inside out.
pub(crate) fn resized(
  low: AnnotationPoint,
  high: AnnotationPoint,
  size: f64,
  edges: u32,
  point: AnnotationPoint,
) -> (AnnotationPoint, f64) {
  let (width, height) = (high.x - low.x, high.y - low.y);
  let along = |moves_low: bool, moves_high: bool, low: f64, high: f64, at: f64, extent: f64| {
    if extent <= 0.0 {
      None
    } else if moves_low {
      Some((high - at) / extent)
    } else if moves_high {
      Some((at - low) / extent)
    } else {
      None
    }
  };
  let ratio_x = along(
    edges & EDGE_LEFT != 0,
    edges & EDGE_RIGHT != 0,
    low.x,
    high.x,
    point.x,
    width,
  );
  let ratio_y = along(
    edges & EDGE_TOP != 0,
    edges & EDGE_BOTTOM != 0,
    low.y,
    high.y,
    point.y,
    height,
  );
  let ratio = match (ratio_x, ratio_y) {
    (Some(x), Some(y)) => x.max(y),
    (Some(ratio), None) | (None, Some(ratio)) => ratio,
    (None, None) => 1.0,
  };
  let next = (size * ratio).max(MIN_STICKER_SIZE.min(size));
  let grown = if size > 0.0 { next / size } else { 1.0 };
  let place = |moves_low: bool, moves_high: bool, low: f64, high: f64, extent: f64| {
    let half = extent * grown * 0.5;
    if moves_low {
      high - half
    } else if moves_high {
      low + half
    } else {
      (low + high) * 0.5
    }
  };
  let center = AnnotationPoint {
    x: place(
      edges & EDGE_LEFT != 0,
      edges & EDGE_RIGHT != 0,
      low.x,
      high.x,
      width,
    ),
    y: place(
      edges & EDGE_TOP != 0,
      edges & EDGE_BOTTOM != 0,
      low.y,
      high.y,
      height,
    ),
  };
  (center, next)
}
