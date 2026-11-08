// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving an image's grips does to it.
//!
//! Its grips are those of its own frame, turned and tilted with the picture,
//! each moving the picture's own sides. A corner or an edge sizes the whole
//! picture, keeping its proportions, with the frame's opposite side held
//! where it was; the body carries it.
//! Every sample is measured against the image the press began on, as it is
//! drawn. A tilted picture is seen from in front of its middle, from a
//! distance that grows with it, so sizing it scales its drawn shape about
//! any of its points: the held point stays put on screen, and each moved
//! side's drawn line is scaled out to the hand.

use super::model::{field_box, half_extents, MIN_IMAGE_SIZE};
use super::tilt::placed;
use super::ImageTilt;
use crate::box_gesture::{carry, EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP};
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
  let (half_width, half_height) = half_extents(from_size, aspect);
  let AnnotationShape::Image { center, size, .. } = &mut annotation.shape else {
    return SnapResult::default();
  };
  match handle {
    AnnotationHandle::Edges(edges) => {
      // The press's picture as drawn, its own point given as shares of its
      // half width and half height from its middle.
      let drawn = |x: f64, y: f64| {
        placed(
          from_center,
          from_angle,
          tilt,
          from_size,
          AnnotationPoint {
            x: x * half_width,
            y: y * half_height,
          },
        )
      };
      (*center, *size) = resized(drawn, from_center, from_size, edges, point);
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

/// The middle and longer side that put the moved sides of the picture drawn
/// by `drawn` under `point`, with the opposite side held where it is drawn.
/// `drawn` places the picture's own point, as shares of its half width and
/// half height from its middle; `center` is its middle and `size` its longer
/// side. A corner takes whichever of its two sides asks for the larger
/// picture, so the frame always reaches the hand; a side dragged past its
/// opposite stops at the smallest image rather than turning it inside out.
pub(crate) fn resized(
  drawn: impl Fn(f64, f64) -> AnnotationPoint,
  center: AnnotationPoint,
  size: f64,
  edges: u32,
  point: AnnotationPoint,
) -> (AnnotationPoint, f64) {
  let along = |low: u32, high: u32| {
    if edges & low != 0 {
      -1.0
    } else if edges & high != 0 {
      1.0
    } else {
      0.0
    }
  };
  let (x, y) = (along(EDGE_LEFT, EDGE_RIGHT), along(EDGE_TOP, EDGE_BOTTOM));
  let held = drawn(-x, -y);
  let offset = |at: AnnotationPoint| (at.x - held.x, at.y - held.y);
  let cross = |a: (f64, f64), b: (f64, f64)| a.0 * b.1 - a.1 * b.0;
  // The drawn picture scaled by `ratio` about `held` puts the side from
  // `from` to `to` on the line through `held + ratio * (from - held)` along
  // `to - from`; this is the ratio that line passes through the hand at.
  let side = |from: AnnotationPoint, to: AnnotationPoint| {
    let start = offset(from);
    let run = (to.x - from.x, to.y - from.y);
    let across = cross(start, run);
    // Seen edge on, the side runs through `held`, and no scale moves it.
    let reach = start.0.hypot(start.1) * run.0.hypot(run.1);
    if across.abs() <= reach * 1e-9 {
      return None;
    }
    Some(cross(offset(point), run) / across).filter(|ratio| ratio.is_finite())
  };
  let ratio_x = (x != 0.0)
    .then(|| side(drawn(x, -1.0), drawn(x, 1.0)))
    .flatten();
  let ratio_y = (y != 0.0)
    .then(|| side(drawn(-1.0, y), drawn(1.0, y)))
    .flatten();
  let ratio = match (ratio_x, ratio_y) {
    (Some(x), Some(y)) => x.max(y),
    (Some(ratio), None) | (None, Some(ratio)) => ratio,
    (None, None) => 1.0,
  };
  let next = (size * ratio).max(MIN_IMAGE_SIZE.min(size));
  let grown = if size > 0.0 { next / size } else { 1.0 };
  let middle = AnnotationPoint {
    x: held.x + (center.x - held.x) * grown,
    y: held.y + (center.y - held.y) * grown,
  };
  (middle, next)
}
