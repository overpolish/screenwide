// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving one of an arrow's grips does to its shape.

use super::bend::{clamp_bend, control_for_bend, control_through_midpoint};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Move one grip of an arrow to `point`. `shift` is unread: an arrow's grips
/// have nothing to quantise to, where a counter's tail has its eighth turns.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  _shift: bool,
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
      let tip = result.tip(point, snap);
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
    // counter's tail grip means nothing to an arrow and moves it likewise.
    AnnotationHandle::Body | AnnotationHandle::Tail => {
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
