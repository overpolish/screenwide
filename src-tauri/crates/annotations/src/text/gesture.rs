// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a text box's grips does to it.

use super::geometry::text_corner;
use super::look::{self, box_point_at};
use super::model::TextControlPoint;
use super::snap::text_box;
use crate::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::snap::{SnapBox, SnapOffset, SnapRequest, SnapResult};
use crate::{Annotation, AnnotationPoint, AnnotationShape};
use screenwide_text::{BoxPoint, ControlSpace};

/// How far past the box a tail reaches, in ems, until it is pulled past the
/// stretch point: every box's tail is the same short tooltip point unless
/// one is deliberately drawn out to something.
const STANDARD_REACH: f64 = 0.55;
/// Where the tail starts following the hand, in screen points past the box.
/// Measured on screen so zooming in brings it closer in the picture, down to
/// the standard reach itself, and any length can be drawn there.
const STRETCH_POINTS: f64 = 24.0;
/// The stretch point in ems where the picture's size on screen is unknown.
const STRETCH_FROM: f64 = 2.0;

/// The reach past which the tail follows the hand, in ems of `em` source
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
/// its only grip is its look's tail tip, and everything else carries the
/// whole box, a tail held against it with it.
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
    text,
    look,
    controls,
    ..
  } = &mut annotation.shape
  else {
    return SnapResult::default();
  };
  if handle == AnnotationHandle::Tail {
    let Some(tail) = look::tail(look, controls) else {
      return SnapResult::default();
    };
    let (id, space, limits) = (
      tail.control.id.clone(),
      tail.control.space,
      tail.control.limits,
    );
    // The tip lands on an element's edge the way an arrow's does, when it is
    // drawn out far enough to point at one.
    let mut result = SnapResult::default();
    let snapped = result.tip(point, snap);
    if space == ControlSpace::Picture {
      controls.insert(id, TextControlPoint::Picture(snapped));
      return result;
    }
    let bounds = text_box(*corner, text, look, width, origin.source_per_size);
    let em = look::type_size(look, width) * origin.source_per_size;
    let from = stretch_from(em, origin.source_per_point);
    let (held, stretched) = pointer_at(bounds, em, from, snapped);
    let (held, result) = if stretched {
      (held, result)
    } else {
      (pointer_at(bounds, em, from, point).0, SnapResult::default())
    };
    controls.insert(id, TextControlPoint::Box(held.within(&limits)));
    return result;
  }
  let AnnotationShape::Text { origin: from, .. } = &origin.shape else {
    return SnapResult::default();
  };
  let travel = SnapOffset {
    x: point.x - origin.point.x,
    y: point.y - origin.point.y,
  };
  let moved = travel.apply(*from);
  // What snaps is the box the drag arrives at, and a tail's tip that moves
  // with it, not the hand: the box may be held anywhere.
  match snap {
    Some(request) => {
      let source_per_size = request.field.source_per_size();
      let bounds = text_box(moved, text, look, width, source_per_size);
      let em = look::type_size(look, width) * source_per_size;
      let tip =
        tail_tip(bounds, em, look, controls).filter(|_| tail_moves_with_box(look, controls));
      let (offset, result) = request.boxed(bounds, tip);
      *corner = offset.apply(moved);
      result
    }
    None => {
      *corner = moved;
      SnapResult::default()
    }
  }
}

/// Whether a look's tail tip is carried along when its box moves: one held
/// against the box is, one put on the picture stays where it was put.
fn tail_moves_with_box(
  look: &screenwide_text::TextStyle,
  controls: &std::collections::BTreeMap<String, TextControlPoint>,
) -> bool {
  matches!(
    look::tail(look, controls).map(|tail| tail.point),
    Some(TextControlPoint::Box(_))
  )
}

/// The box point that puts a tail's tip at `point`, against `bounds` with
/// its type `em` source pixels high, and whether it is drawn out past
/// `stretch_from` ems. A tip inside the box tucks the tail in on the nearest
/// stretch of edge, clear of the rounded corners, so its grip always sits on
/// the outline; one just past the edge is the standard tooltip point, in the
/// direction it was pulled; only one pulled further stretches to where it
/// was pulled.
fn pointer_at(
  bounds: SnapBox,
  em: f64,
  stretch_from: f64,
  point: AnnotationPoint,
) -> (BoxPoint, bool) {
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
    BoxPoint {
      along: [along_x, along_y],
      reach: [reach_x, reach_y],
    },
    stretched,
  )
}

/// Where a drawn tail's tip is in the picture against `bounds`, for it to
/// take an element's edge when the box is moved; `None` when it is tucked
/// in or the look draws none.
pub(crate) fn tail_tip(
  bounds: SnapBox,
  em: f64,
  look: &screenwide_text::TextStyle,
  controls: &std::collections::BTreeMap<String, TextControlPoint>,
) -> Option<AnnotationPoint> {
  let tail = look::tail(look, controls).filter(|tail| tail.is_out())?;
  Some(match tail.point {
    TextControlPoint::Box(point) => box_point_at(
      AnnotationPoint {
        x: bounds.x,
        y: bounds.y,
      },
      [bounds.width, bounds.height],
      em,
      &point,
    ),
    TextControlPoint::Picture(point) => point,
  })
}
