// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving an image's grips does to it.
//!
//! Its grips are those of its own frame, turned and tilted with the picture,
//! each moving the picture's own sides. A corner or an edge sizes the whole
//! picture, keeping its proportions, with the frame's opposite side held
//! where it was; the radius dot rounds its corners; the body carries it.
//! Every sample is measured against the image the press began on, in that
//! image's own frame, the hand carried back through its tilt.

use super::model::{field_box, half_extents, MIN_IMAGE_SIZE};
use super::tilt::{placed, unplaced};
use super::ImageTilt;
use crate::box_gesture::{carry, radius_at, EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP};
use crate::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::snap::{SnapBox, SnapRequest, SnapResult};
use crate::{Annotation, AnnotationPoint, AnnotationShape};

/// The image a gesture began on: its middle, longer side, flat turn, turn in
/// depth and proportions.
fn held(shape: &AnnotationShape) -> Option<(AnnotationPoint, f64, f64, ImageTilt, f64)> {
  match shape {
    AnnotationShape::Image {
      center,
      size,
      angle,
      tilt,
      aspect,
      ..
    } => Some((*center, *size, *angle, tilt.unwrap_or_default(), *aspect)),
    _ => None,
  }
}

/// Move one grip of an image to `point`.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let Some((from_center, from_size, from_angle, tilt, aspect)) = held(&origin.shape) else {
    return SnapResult::default();
  };
  // The press's image in its own frame: its middle at the origin, its sides
  // upright. A hand past the tilted picture's horizon reaches no side of it.
  let (half_width, half_height) = half_extents(from_size, aspect);
  let local = unplaced(from_center, from_angle, tilt, from_size, point);
  if handle == AnnotationHandle::Radius {
    let Some(local) = local else {
      return SnapResult::default();
    };
    let own = SnapBox {
      x: -half_width,
      y: -half_height,
      width: half_width * 2.0,
      height: half_height * 2.0,
    };
    annotation.style.radius = radius_at(own, local, origin.source_per_point);
    return SnapResult::default();
  }
  let AnnotationShape::Image { center, size, .. } = &mut annotation.shape else {
    return SnapResult::default();
  };
  match handle {
    AnnotationHandle::Edges(edges) => {
      let Some(local) = local else {
        return SnapResult::default();
      };
      let low = AnnotationPoint {
        x: -half_width,
        y: -half_height,
      };
      let high = AnnotationPoint {
        x: half_width,
        y: half_height,
      };
      // The middle moves within the picture's own plane, to where its tilt
      // draws it there; the new picture is tilted about it alike.
      let (middle, next) = resized(low, high, from_size, edges, local);
      (*center, *size) = (
        placed(from_center, from_angle, tilt, from_size, middle),
        next,
      );
      SnapResult::default()
    }
    _ => {
      let from = field_box(from_center, from_size, from_angle, tilt, aspect).unwrap_or(SnapBox {
        x: from_center.x,
        y: from_center.y,
        width: 0.0,
        height: 0.0,
      });
      // The box carries the middle with it; a tilt sets the box off the
      // middle, so the middle travels as far rather than to the box's.
      let (moved, result) = carry(from, point, origin.point, snap);
      *center = AnnotationPoint {
        x: from_center.x + moved.x - from.x,
        y: from_center.y + moved.y - from.y,
      };
      result
    }
  }
}

/// A fresh image is dropped whole where the press lands, and the same drag
/// carries it; what snaps is the box it fits in, not the pointer.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let AnnotationShape::Image {
    center,
    size,
    angle,
    tilt,
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
  let Some(dropped) = field_box(point, *size, *angle, tilt.unwrap_or_default(), *aspect) else {
    *center = point;
    return SnapResult::default();
  };
  let (offset, result) = request.boxed(dropped, None);
  let moved = dropped.moved(offset);
  *center = AnnotationPoint {
    x: point.x + moved.x - dropped.x,
    y: point.y + moved.y - dropped.y,
  };
  result
}

/// The middle and longer side that put the box's moved sides under `point`
/// with its opposite sides held. A corner takes whichever of its two sides
/// asks for the larger picture, so the box always reaches the hand; a side
/// dragged past its opposite stops at the smallest image rather than
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
  let next = (size * ratio).max(MIN_IMAGE_SIZE.min(size));
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
