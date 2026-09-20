// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a counter's grips does to it.

use super::model::counter_tail_angle;
use super::silhouette::counter_tail_tip;
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Move one grip of a counter to `point`. `shift` holds the tail to the
/// eighth turns.
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
  let width = annotation.style.width;
  let AnnotationShape::Counter { center, angle, .. } = &mut annotation.shape else {
    return result;
  };
  match handle {
    // The tail turns around the disc: the drag sets its direction and
    // nothing else, so a counter cannot be stretched out of shape. The
    // aim the hand gives comes first, because an element edge within
    // reach of it beats both that aim and Shift's eighth turns.
    AnnotationHandle::Tail => {
      let aimed = counter_tail_angle(*center, point, *angle, false);
      match snap.and_then(|request| request.tail(*center, request.field.radius(width), aimed)) {
        Some((edge, anchor)) => {
          *angle = edge;
          result.anchor = Some(anchor);
        }
        None => *angle = counter_tail_angle(*center, point, *angle, shift),
      }
    }
    // Everything else carries the whole counter, the disc included: a
    // grip an arrow has and a counter does not is a move rather than
    // nothing at all.
    _ => {
      let AnnotationShape::Counter { center: from, .. } = origin.shape else {
        return result;
      };
      let moved = AnnotationPoint {
        x: from.x + point.x - origin.point.x,
        y: from.y + point.y - origin.point.y,
      };
      // What snaps is the disc the drag arrives at, not the pointer: the
      // grip may be anywhere on it, and a counter lines up by its edges
      // as readily as by its middle. Its tail tip travels with it and
      // competes for an element's edges in either axis.
      *center = match snap {
        Some(request) => {
          let radius = request.field.radius(width);
          let (offset, resolved) = request.counter(
            request.field.disc(moved, width),
            counter_tail_tip(moved, radius, *angle),
          );
          result = resolved;
          offset.apply(moved)
        }
        None => moved,
      };
    }
  }
  result
}
