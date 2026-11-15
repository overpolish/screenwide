// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving one of an arrow's grips does to its shape.

use super::bend::{clamp_bend, control_for_bend, control_through_midpoint};
use crate::counter::model::COUNTER_SNAP_RADIANS;
use crate::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::snap::{SnapRequest, SnapResult};
use crate::{Annotation, AnnotationPoint, AnnotationShape};

/// Where a tip dragged to `point` lands with Shift held: on the nearest of
/// the eight directions from `fixed`, the other tip, a counter's tail's eighth
/// turns, at the distance the hand is from `fixed`. A drag lands on a line
/// held level, upright or diagonal however the hand wavers.
pub(crate) fn on_eighth_turn(fixed: AnnotationPoint, point: AnnotationPoint) -> AnnotationPoint {
  let (dx, dy) = (point.x - fixed.x, point.y - fixed.y);
  let reach = dx.hypot(dy);
  if reach == 0.0 {
    return point;
  }
  let angle = (dy.atan2(dx) / COUNTER_SNAP_RADIANS).round() * COUNTER_SNAP_RADIANS;
  AnnotationPoint {
    x: fixed.x + reach * angle.cos(),
    y: fixed.y + reach * angle.sin(),
  }
}

/// Move one grip of an arrow to `point`. `shift` holds a tip to the eighth
/// turns about the other tip, in place of snapping it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
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
  match handle {
    // A tip takes the bend with it: the curve is re-hung from the chord the
    // drag leaves behind, holding the share of it the arrow was bent by, so
    // it rotates and scales with the shaft and can never be stranded. The tip
    // is what an arrow aims with, so it is the only part of one that snaps,
    // and the bend is re-hung from wherever it lands.
    AnnotationHandle::Start | AnnotationHandle::End => {
      let fixed = if handle == AnnotationHandle::Start {
        *end
      } else {
        *start
      };
      let tip = if shift {
        on_eighth_turn(fixed, point)
      } else {
        result.tip(point, snap)
      };
      if handle == AnnotationHandle::Start {
        *start = tip;
      } else {
        *end = tip;
      }
      *control = control_for_bend(*start, *end, origin.bend);
    }
    AnnotationHandle::Middle => *control = control_through_midpoint(*start, point, *end),
    // The shaft carries the arrow whole: every point travels by the same
    // delta, so the curve keeps its bend and its heads keep their aim. A
    // counter's tail grip or a box's grips mean nothing to an arrow and move
    // it likewise.
    AnnotationHandle::Body | AnnotationHandle::Tail | AnnotationHandle::Edges(_) => {
      let AnnotationShape::Arrow {
        start: from_start,
        control: from_control,
        end: from_end,
      } = origin.shape
      else {
        return result;
      };
      let delta_x = point.x - origin.point.x;
      let delta_y = point.y - origin.point.y;
      let moved = |point: AnnotationPoint| AnnotationPoint {
        x: point.x + delta_x,
        y: point.y + delta_y,
      };
      *start = moved(from_start);
      *control = moved(from_control);
      *end = moved(from_end);
    }
  }
  // Every edit leaves an arrow that can be drawn: the middle handle can be
  // dragged past a tip, and a document written before the limit existed is
  // repaired the first time its arrow is touched.
  clamp_bend(annotation);
  result
}
