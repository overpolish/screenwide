// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a text box's grips does to it.

use super::geometry::text_corner;
use super::model::TextPointer;
use super::snap::text_box;
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapBox, SnapOffset, SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// How far past the box a pointer reaches, in ems, until it is pulled past
/// the stretch point: every box's pointer is the same short tooltip point
/// unless one is deliberately drawn out to something.
const STANDARD_REACH: f64 = 0.55;
/// Where the pointer starts following the hand, in screen points past the
/// box. Measured on screen so zooming in brings it closer in the picture,
/// down to the standard reach itself, and any length can be drawn there.
const STRETCH_POINTS: f64 = 24.0;
/// The stretch point in ems where the picture's size on screen is unknown.
const STRETCH_FROM: f64 = 2.0;

/// The reach past which the pointer follows the hand, in ems of `em` source
/// pixels, for a picture drawn at `source_per_point` source pixels per
/// screen point.
fn stretch_from(em: f64, source_per_point: f64) -> f64 {
  if em > 0.0 && source_per_point > 0.0 {
    (STRETCH_POINTS * source_per_point / em).max(STANDARD_REACH)
  } else {
    STRETCH_FROM
  }
}

/// Move one grip of a text box to `point`. The box itself is never resized:
/// its only grip is the pointer's tip, and everything else carries the whole
/// box, its pointer with it.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let width = annotation.style.width;
  let AnnotationShape::Text {
    origin: corner,
    pointer,
    text,
  } = &mut annotation.shape
  else {
    return SnapResult::default();
  };
  if handle == AnnotationHandle::Tail {
    // The tip lands on an element's edge the way an arrow's does, when it is
    // drawn out far enough to point at one.
    let bounds = text_box(*corner, text, width, origin.source_per_output);
    let em = width.max(0.0) * origin.source_per_output;
    let from = stretch_from(em, origin.source_per_point);
    let mut result = SnapResult::default();
    let snapped = result.tip(point, snap);
    let (held, stretched) = pointer_at(bounds, em, from, snapped);
    if stretched {
      *pointer = held;
      return result;
    }
    *pointer = pointer_at(bounds, em, from, point).0;
    return SnapResult::default();
  }
  let AnnotationShape::Text { origin: from, .. } = &origin.shape else {
    return SnapResult::default();
  };
  let travel = SnapOffset {
    x: point.x - origin.point.x,
    y: point.y - origin.point.y,
  };
  let moved = travel.apply(*from);
  // What snaps is the box the drag arrives at, and its pointer's tip, not
  // the hand: the box may be held anywhere.
  match snap {
    Some(request) => {
      let bounds = text_box(moved, text, width, request.field.source_per_output());
      let em = width.max(0.0) * request.field.source_per_output();
      let (offset, result) = request.boxed(bounds, pointer_tip(bounds, em, pointer));
      *corner = offset.apply(moved);
      result
    }
    None => {
      *corner = moved;
      SnapResult::default()
    }
  }
}

/// The pointer that puts its tip at `point`, against `bounds` with its type
/// `em` source pixels high, and whether it is drawn out past `stretch_from`
/// ems. A tip inside the box tucks the pointer in on the nearest stretch of
/// edge, clear of the rounded corners, so its grip always sits on the
/// outline; one just past the edge is the standard tooltip point, in the
/// direction it was pulled; only one pulled further stretches to where it
/// was pulled.
fn pointer_at(
  bounds: SnapBox,
  em: f64,
  stretch_from: f64,
  point: AnnotationPoint,
) -> (TextPointer, bool) {
  let axis = |point: f64, start: f64, extent: f64| {
    let half = extent / 2.0;
    let offset = point - (start + half);
    if half <= 0.0 {
      return (0.0, 0.0);
    }
    if offset.abs() <= half {
      return (offset / half, 0.0);
    }
    let past = if em > 0.0 {
      (offset.abs() - half) / em
    } else {
      0.0
    };
    (offset.signum(), past)
  };
  let (mut along_x, mut reach_x) = axis(point.x, bounds.x, bounds.width);
  let (mut along_y, mut reach_y) = axis(point.y, bounds.y, bounds.height);
  if reach_x == 0.0 && reach_y == 0.0 {
    let (half_x, half_y) = (bounds.width / 2.0, bounds.height / 2.0);
    let corner = text_corner(em).min(half_x).min(half_y);
    let straight = |half: f64| if half > 0.0 { 1.0 - corner / half } else { 0.0 };
    // The edge it is nearest, in source pixels, is the one it tucks into.
    if half_x * (1.0 - along_x.abs()) <= half_y * (1.0 - along_y.abs()) {
      along_x = if along_x < 0.0 { -1.0 } else { 1.0 };
      along_y = along_y.clamp(-straight(half_y), straight(half_y));
    } else {
      along_y = if along_y < 0.0 { -1.0 } else { 1.0 };
      along_x = along_x.clamp(-straight(half_x), straight(half_x));
    }
  }
  let reach = reach_x.hypot(reach_y);
  let stretched = reach > stretch_from;
  if reach > 0.0 && !stretched {
    reach_x *= STANDARD_REACH / reach;
    reach_y *= STANDARD_REACH / reach;
  }
  (
    TextPointer {
      along: AnnotationPoint {
        x: along_x,
        y: along_y,
      },
      reach: AnnotationPoint {
        x: reach_x,
        y: reach_y,
      },
    },
    stretched,
  )
}

/// Where a drawn pointer's tip is against `bounds`, for it to take an
/// element's edge when the box is moved; `None` when it is tucked in.
fn pointer_tip(bounds: SnapBox, em: f64, pointer: &TextPointer) -> Option<AnnotationPoint> {
  if !pointer.is_drawn() {
    return None;
  }
  let axis = |start: f64, extent: f64, along: f64, reach: f64| {
    let half = extent / 2.0;
    start + half + along * half + (reach * em).copysign(along)
  };
  Some(AnnotationPoint {
    x: axis(bounds.x, bounds.width, pointer.along.x, pointer.reach.x),
    y: axis(bounds.y, bounds.height, pointer.along.y, pointer.reach.y),
  })
}
