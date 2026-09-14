// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an arrow gesture acts on, and what moving one grip does to the shape.
//!
//! The native interaction view reports a grip by number; everything that
//! decides what an arrow *is* lives on this side, so the native side never
//! carries a second copy of the model.

use super::bend::{arrow_bend, clamp_bend, control_for_bend, control_through_midpoint, ArrowBend};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Which grip of an arrow the pointer took hold of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationHandle {
  Start,
  Middle,
  End,
  /// The shaft. Dragging it carries the whole arrow along.
  Body,
}

impl AnnotationHandle {
  pub(crate) fn from_raw(value: u32) -> Option<Self> {
    match value {
      0 => Some(Self::Start),
      1 => Some(Self::Middle),
      2 => Some(Self::End),
      3 => Some(Self::Body),
      _ => None,
    }
  }
}

/// What the gesture acts on: an arrow being drawn, or one already there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationGestureTarget {
  NewArrow,
  Existing {
    index: usize,
    handle: AnnotationHandle,
  },
  /// A press that landed on no arrow at all, with only the select tool in
  /// hand. It lets the chosen arrow go and then belongs to the layer.
  None,
  /// A press on the shaft of the arrow at `index`. It only chooses that
  /// arrow: the move it may turn into arrives as its own `Existing` gesture
  /// once the press has travelled past the native slop.
  Select {
    index: usize,
  },
}

impl AnnotationGestureTarget {
  /// Reads the target the native interaction view reported: a new arrow (0),
  /// a grip of the arrow at `index` (1), no arrow at all (2), or a press that
  /// only chooses the arrow at `index` (3).
  pub(crate) fn from_raw(kind: u32, index: u32, handle: u32) -> Option<Self> {
    match kind {
      0 => Some(Self::NewArrow),
      1 => Some(Self::Existing {
        index: index as usize,
        handle: AnnotationHandle::from_raw(handle)?,
      }),
      2 => Some(Self::None),
      3 => Some(Self::Select {
        index: index as usize,
      }),
      _ => None,
    }
  }
}

/// Where a drag started, the shape it started from, and how that shape was
/// bent against its own chord.
///
/// A whole-arrow move is expressed against the shape the press began on
/// rather than against the last sample, so a drag that is nudged back and
/// forth lands exactly where the pointer is instead of accumulating the
/// rounding of every frame in between. The bend is held the same way, and in
/// the chord's terms, so dragging a tip carries the curve round with the
/// shaft rather than leaving the control point behind in the canvas.
#[derive(Clone, Debug)]
pub(crate) struct AnnotationDragOrigin {
  pub(crate) bend: ArrowBend,
  pub(crate) point: AnnotationPoint,
  pub(crate) shape: AnnotationShape,
}

impl AnnotationDragOrigin {
  pub(crate) fn new(point: AnnotationPoint, shape: &AnnotationShape) -> Self {
    let AnnotationShape::Arrow {
      start,
      control,
      end,
    } = shape;
    Self {
      bend: arrow_bend(*start, *control, *end).clamped(),
      point,
      shape: shape.clone(),
    }
  }
}

/// Names a fresh arrow. Collisions only have to be impossible inside one
/// document, and a monotonic counter beside the clock gives that without
/// reaching for a dependency.
pub(crate) fn next_annotation_id() -> String {
  use std::sync::atomic::{AtomicU64, Ordering};
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_millis() as u64);
  format!("arrow-{millis:x}-{sequence:x}")
}

/// Move one grip of an arrow to `point`, in source pixels. `origin` is where
/// the drag began, which is what the shaft measures its travel against.
pub(crate) fn drag_handle(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
) {
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = &mut annotation.shape;
  match handle {
    // A tip takes the bend with it: the curve is re-hung from the chord the
    // drag leaves behind, holding the share of it the arrow was bent by, so
    // it rotates and scales with the shaft and can never be stranded.
    AnnotationHandle::Start | AnnotationHandle::End => {
      if handle == AnnotationHandle::Start {
        *start = point;
      } else {
        *end = point;
      }
      *control = control_for_bend(*start, *end, origin.bend);
    }
    AnnotationHandle::Middle => *control = control_through_midpoint(*start, point, *end),
    // The shaft carries the arrow whole: every point travels by the same
    // delta, so the curve keeps its bend and its heads keep their aim.
    AnnotationHandle::Body => {
      let AnnotationShape::Arrow {
        start: from_start,
        control: from_control,
        end: from_end,
      } = origin.shape;
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
}
