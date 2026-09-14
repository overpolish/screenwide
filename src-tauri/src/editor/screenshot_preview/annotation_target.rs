// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an arrow gesture acts on, and what moving one grip does to the shape.
//!
//! The native interaction view reports a grip by number; everything that
//! decides what an arrow *is* lives on this side, so the native side never
//! carries a second copy of the model.

use super::annotation::control_through_midpoint;
use crate::screenshots::{Annotation, AnnotationPoint, AnnotationShape};

/// Which grip of an arrow the pointer took hold of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationHandle {
  Start,
  Middle,
  End,
  /// The shaft. A press there only selects the arrow.
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
}

impl AnnotationGestureTarget {
  /// Reads the target the native interaction view reported: a new arrow (0),
  /// a grip of the arrow at `index` (1), or no arrow at all (2).
  pub(crate) fn from_raw(kind: u32, index: u32, handle: u32) -> Option<Self> {
    match kind {
      0 => Some(Self::NewArrow),
      1 => Some(Self::Existing {
        index: index as usize,
        handle: AnnotationHandle::from_raw(handle)?,
      }),
      2 => Some(Self::None),
      _ => None,
    }
  }
}

/// Names a fresh arrow. Collisions only have to be impossible inside one
/// document, and a monotonic counter beside the clock gives that without
/// reaching for a dependency.
pub(super) fn next_annotation_id() -> String {
  use std::sync::atomic::{AtomicU64, Ordering};
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_millis() as u64);
  format!("arrow-{millis:x}-{sequence:x}")
}

/// Move one grip of an arrow to `point`, in source pixels.
pub(super) fn drag_handle(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
) {
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = &mut annotation.shape;
  match handle {
    // The bend is carried by the control point, so a tip keeps it as it is
    // and the curve simply re-hangs from the moved end.
    AnnotationHandle::Start => *start = point,
    AnnotationHandle::End => *end = point,
    AnnotationHandle::Middle => *control = control_through_midpoint(*start, point, *end),
    AnnotationHandle::Body => {}
  }
}
