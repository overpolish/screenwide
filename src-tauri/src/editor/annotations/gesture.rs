// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an arrow gesture acts on, and what moving one grip does to the shape.
//!
//! The native interaction view reports a grip by number; everything that
//! decides what an arrow *is* lives on this side, so the native side never
//! carries a second copy of the model.

use super::bend::{arrow_bend, clamp_bend, control_for_bend, control_through_midpoint, ArrowBend};
use super::counter::counter_tail_angle;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Which grip of a mark the pointer took hold of. An arrow has three grips
/// and its shaft; a counter has one - the tail - and its disc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationHandle {
  Start,
  Middle,
  End,
  /// The shaft, or a counter's disc. Dragging it carries the whole mark.
  Body,
  /// A counter's tail tip. Dragging it turns the tail around the disc.
  Tail,
}

impl AnnotationHandle {
  pub(crate) fn from_raw(value: u32) -> Option<Self> {
    match value {
      0 => Some(Self::Start),
      1 => Some(Self::Middle),
      2 => Some(Self::End),
      3 => Some(Self::Body),
      4 => Some(Self::Tail),
      _ => None,
    }
  }
}

/// What the gesture acts on: a mark being drawn, or one already there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationGestureTarget {
  /// Empty picture under a drawing tool. Which shape it makes is the tool's
  /// business rather than the native view's, so it rides in beside the
  /// target as [`NewMarkKind`].
  New,
  Existing {
    index: usize,
    handle: AnnotationHandle,
  },
  /// A press that landed on no mark at all, with only the select tool in
  /// hand. It lets the chosen mark go and then belongs to the layer.
  None,
  /// A press on the body of the mark at `index`. It only chooses that mark:
  /// the move it may turn into arrives as its own `Existing` gesture once
  /// the press has travelled past the native slop.
  Select { index: usize },
}

/// Which shape a drawing tool's press makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewMarkKind {
  Arrow,
  Counter,
}

/// What the pointer does over the picture while a tool is in hand. The select
/// tool hit-tests the marks already there and lets every other press fall
/// through to the layer; a drawing tool also makes a new mark on empty
/// picture. The values are the native `ScreenwideAnnotationMode`.
pub(crate) const MODE_NONE: u32 = 0;
pub(crate) const MODE_SELECT: u32 = 1;
pub(crate) const MODE_ARROW: u32 = 2;
pub(crate) const MODE_COUNTER: u32 = 3;

/// The tool name React sends, as a mode. Anything else puts the chrome away.
pub(crate) fn annotation_mode(tool: Option<&str>) -> u32 {
  match tool {
    Some("arrow") => MODE_ARROW,
    Some("counter") => MODE_COUNTER,
    Some("select") => MODE_SELECT,
    _ => MODE_NONE,
  }
}

impl NewMarkKind {
  /// The shape the tool in hand draws. Only the drawing modes make a mark at
  /// all, so anything else answers the arrow it would have drawn.
  pub(crate) fn from_mode(mode: u32) -> Self {
    if mode == MODE_COUNTER {
      Self::Counter
    } else {
      Self::Arrow
    }
  }
}

impl AnnotationGestureTarget {
  /// Reads the target the native interaction view reported: a new mark (0),
  /// a grip of the mark at `index` (1), no mark at all (2), or a press that
  /// only chooses the mark at `index` (3).
  pub(crate) fn from_raw(kind: u32, index: u32, handle: u32) -> Option<Self> {
    match kind {
      0 => Some(Self::New),
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
    Self {
      // A counter has no chord to be bent against; the default bend is never
      // read for one.
      bend: match shape {
        AnnotationShape::Arrow {
          start,
          control,
          end,
        } => arrow_bend(*start, *control, *end).clamped(),
        AnnotationShape::Counter { .. } => ArrowBend::STRAIGHT,
      },
      point,
      shape: shape.clone(),
    }
  }
}

/// Names a fresh mark. Collisions only have to be impossible inside one
/// document, and a monotonic counter beside the clock gives that without
/// reaching for a dependency.
pub(crate) fn next_annotation_id() -> String {
  use std::sync::atomic::{AtomicU64, Ordering};
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_millis() as u64);
  format!("mark-{millis:x}-{sequence:x}")
}

/// Move one grip of a mark to `point`, in source pixels. `origin` is where
/// the drag began, which is what a whole-mark move measures its travel
/// against, and `snap` is whether Shift was held - which holds a counter's
/// tail to the quarter turns.
pub(crate) fn drag_handle(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  snap: bool,
) {
  match &mut annotation.shape {
    AnnotationShape::Arrow {
      start,
      control,
      end,
    } => drag_arrow_handle(start, control, end, handle, point, origin),
    AnnotationShape::Counter { center, angle, .. } => {
      match handle {
        // The tail turns around the disc: the drag sets its direction and
        // nothing else, so a counter cannot be stretched out of shape.
        AnnotationHandle::Tail => *angle = counter_tail_angle(*center, point, *angle, snap),
        // Everything else carries the whole counter, the disc included: a
        // grip an arrow has and a counter does not is a move rather than
        // nothing at all.
        _ => {
          let AnnotationShape::Counter { center: from, .. } = origin.shape else {
            return;
          };
          *center = AnnotationPoint {
            x: from.x + point.x - origin.point.x,
            y: from.y + point.y - origin.point.y,
          };
        }
      }
      return;
    }
  }
  // Every edit leaves an arrow that can be drawn: the middle handle can be
  // dragged past a tip, and a document written before the limit existed is
  // repaired the first time its arrow is touched.
  clamp_bend(annotation);
}

fn drag_arrow_handle(
  start: &mut AnnotationPoint,
  control: &mut AnnotationPoint,
  end: &mut AnnotationPoint,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
) {
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
    // delta, so the curve keeps its bend and its heads keep their aim. A
    // counter's tail grip means nothing to an arrow and moves it likewise.
    AnnotationHandle::Body | AnnotationHandle::Tail => {
      let AnnotationShape::Arrow {
        start: from_start,
        control: from_control,
        end: from_end,
      } = origin.shape
      else {
        return;
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
}
