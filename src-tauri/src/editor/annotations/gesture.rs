// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a gesture acts on: which grip the pointer took hold of, which tool
//! is in hand, and the shape a drag began from.
//!
//! The native interaction view reports a grip by number; everything that
//! decides what an annotation *is* lives on this side, so the native side
//! never carries a second copy of the model. What a grip does to each shape
//! is that kind's own, in its `gesture` module.

use super::arrow::bend::ArrowBend;
use crate::editor::annotations::{AnnotationKind, AnnotationPoint, AnnotationShape};

/// Which grip of an annotation the pointer took hold of. An arrow has three
/// grips and its shaft; a counter has one - the tail - and its disc; a text
/// box has one - its pointer's tip - and its box; a redaction has the eight
/// grips of a box, its radius dot and its body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationHandle {
  Start,
  Middle,
  End,
  /// The shaft, a counter's disc, a text box or a redaction's body.
  /// Dragging it carries the whole annotation.
  Body,
  /// A counter's tail tip, which turns the tail around the disc, or a text
  /// box's pointer tip, which draws the pointer out of the box or pushes it
  /// back in.
  Tail,
  /// A redaction's corner or edge grip: which of the box's sides it moves,
  /// as the `redact::gesture::EDGE_*` bits.
  Edges(u32),
  /// A redaction's radius dot, which rounds its corners.
  Radius,
}

impl AnnotationHandle {
  pub(crate) fn from_raw(value: u32) -> Option<Self> {
    match value {
      0 => Some(Self::Start),
      1 => Some(Self::Middle),
      2 => Some(Self::End),
      3 => Some(Self::Body),
      4 => Some(Self::Tail),
      // A box grip is its side bits past `BOX_HANDLES`. Opposite sides never
      // move together, and a grip always moves at least one.
      raw if (BOX_HANDLES..BOX_HANDLES + 16).contains(&raw) => {
        let edges = raw - BOX_HANDLES;
        let opposed = |pair: u32| edges & pair == pair;
        (edges != 0 && !opposed(0b0011) && !opposed(0b1100)).then_some(Self::Edges(edges))
      }
      RADIUS_HANDLE => Some(Self::Radius),
      _ => None,
    }
  }
}

/// Where the native `ScreenwideAnnotationHandleBox` grips start: a box grip
/// reports this plus the bits of the sides it moves.
pub(crate) const BOX_HANDLES: u32 = 16;
/// The native `ScreenwideAnnotationHandleRadius`, past every box grip.
pub(crate) const RADIUS_HANDLE: u32 = BOX_HANDLES + 16;

/// What the gesture acts on: an annotation being drawn, or one already there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnotationGestureTarget {
  /// Empty picture under a drawing tool. Which shape it makes is the tool's
  /// business rather than the native view's, so it rides in beside the
  /// target as the mode's [`AnnotationKind`].
  New,
  Existing {
    index: usize,
    handle: AnnotationHandle,
  },
  /// A press that landed on no annotation at all, with only the select tool in
  /// hand. It lets the chosen annotation go and then belongs to the layer.
  None,
  /// A press on the body of the annotation at `index`. It only chooses that
  /// annotation: the move it may turn into arrives as its own `Existing`
  /// gesture once the press has travelled past the native slop.
  Select { index: usize },
}

/// What the pointer does over the picture while a tool is in hand. The select
/// tool hit-tests the annotations already there and lets every other press fall
/// through to the layer; a drawing tool also makes a new annotation on empty
/// picture. The values are the native `ScreenwideAnnotationMode`.
pub(crate) const MODE_NONE: u32 = 0;
pub(crate) const MODE_SELECT: u32 = 1;
pub(crate) const MODE_ARROW: u32 = 2;
pub(crate) const MODE_COUNTER: u32 = 3;
pub(crate) const MODE_TEXT: u32 = 4;
pub(crate) const MODE_REDACT: u32 = 5;

/// The tool name React sends, as a mode. Anything else puts the chrome away.
pub(crate) fn annotation_mode(tool: Option<&str>) -> u32 {
  match tool {
    Some("arrow") => MODE_ARROW,
    Some("counter") => MODE_COUNTER,
    Some("text") => MODE_TEXT,
    Some("redact") => MODE_REDACT,
    Some("select") => MODE_SELECT,
    _ => MODE_NONE,
  }
}

/// The shape the tool in hand draws, or `None` where the tool draws nothing.
/// This is also the one answer to "does this mode draw at all": both hosts
/// ask before a press on empty picture may become an annotation, so a
/// [`AnnotationGestureTarget::New`] under any other mode never arrives.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn drawing_kind(mode: u32) -> Option<AnnotationKind> {
  match mode {
    MODE_ARROW => Some(AnnotationKind::Arrow),
    MODE_COUNTER => Some(AnnotationKind::Counter),
    MODE_TEXT => Some(AnnotationKind::Text),
    MODE_REDACT => Some(AnnotationKind::Redact),
    _ => None,
  }
}

impl AnnotationGestureTarget {
  /// Reads the target the native interaction view reported: a new annotation
  /// (0), a grip of the annotation at `index` (1), no annotation at all (2), or
  /// a press that only chooses the annotation at `index` (3).
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
///
/// `source_per_output` turns a size a style carries into source pixels: a
/// text box's pointer is pushed back in when its tip lands inside the box,
/// and the box is only known in output pixels. Zero where the workspace has
/// not said, which leaves every box a point. `source_per_point` is how many
/// source pixels one screen point covers at the current zoom, which a text
/// box's pointer measures its stretch point in; zero where it is unknown.
#[derive(Clone, Debug)]
pub(crate) struct AnnotationDragOrigin {
  pub(crate) bend: ArrowBend,
  pub(crate) point: AnnotationPoint,
  pub(crate) shape: AnnotationShape,
  pub(crate) source_per_output: f64,
  pub(crate) source_per_point: f64,
}

impl AnnotationDragOrigin {
  pub(crate) fn new(point: AnnotationPoint, shape: &AnnotationShape) -> Self {
    Self {
      bend: shape.bend(),
      point,
      shape: shape.clone(),
      source_per_output: 0.0,
      source_per_point: 0.0,
    }
  }
}

/// Names a fresh annotation. Collisions only have to be impossible inside one
/// document, and a monotonic counter beside the clock gives that without
/// reaching for a dependency.
pub(crate) fn next_annotation_id() -> String {
  use std::sync::atomic::{AtomicU64, Ordering};
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_millis() as u64);
  format!("annotation-{millis:x}-{sequence:x}")
}
