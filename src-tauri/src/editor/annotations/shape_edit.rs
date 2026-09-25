// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The per-kind answers a gesture and a clip ask for, beside the ones in
//! [`super`]: together they are the whole list of what a kind has to answer.

use super::super::reveal::AnnotationReveal;
use super::super::{AnnotationKind, AnnotationPoint};

impl super::super::Annotation {
  /// Move one grip to `point`, in source pixels. `origin` is where the drag
  /// began, which is what a whole-annotation move measures its travel
  /// against, and `shift` is whether Shift was held - which holds a
  /// counter's tail to the quarter turns.
  ///
  /// `snap` is the positional candidates this sample may land on, absent
  /// when the positional modifier is not held. A counter's disc and a text
  /// box align to the axis guides and an arrow's tip takes an element anchor;
  /// neither kind ever sees the other's candidates. What it landed on is
  /// reported back for the chrome to draw.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn drag_grip(
    &mut self,
    handle: super::super::gesture::AnnotationHandle,
    point: AnnotationPoint,
    origin: &super::super::gesture::AnnotationDragOrigin,
    shift: bool,
    snap: Option<super::super::snap::SnapRequest<'_>>,
  ) -> super::super::snap::SnapResult {
    match self.shape.kind() {
      AnnotationKind::Arrow => {
        super::super::arrow::gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Counter => {
        super::super::counter::gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Text => super::super::text::gesture::drag(self, handle, point, origin, snap),
      AnnotationKind::Redact => {
        super::super::redact::gesture::drag(self, handle, point, origin, shift, snap)
      }
    }
  }

  /// Carry the annotation this gesture has just made to `point`: an arrow is
  /// drawn out from the press, a counter was dropped whole there, a text
  /// box is carried the way a placed one is moved, so it stays centred under
  /// the hand it was centred on, and a redaction is pulled out from the press
  /// to the hand, square while `shift` is held. `origin` is where the gesture
  /// began.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn drag_new(
    &mut self,
    point: AnnotationPoint,
    origin: &super::super::gesture::AnnotationDragOrigin,
    shift: bool,
    snap: Option<super::super::snap::SnapRequest<'_>>,
  ) -> super::super::snap::SnapResult {
    match self.shape.kind() {
      AnnotationKind::Arrow => super::super::arrow::snap::drag_new(self, point, snap),
      AnnotationKind::Counter => super::super::counter::snap::drag_new(self, point, snap),
      AnnotationKind::Text => super::super::text::gesture::drag(
        self,
        super::super::gesture::AnnotationHandle::Body,
        point,
        origin,
        snap,
      ),
      AnnotationKind::Redact => {
        super::super::redact::gesture::drag_new(self, point, origin, shift, snap)
      }
    }
  }
}

impl AnnotationKind {
  /// The annotation a fresh press of this tool makes at `point`, in `style`
  /// or in the tool's own first dress. `angle` is where a counter's tail
  /// points, `existing` the list it is numbered against, and
  /// `source_per_output` what sizes a text box so it can centre on `point`.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn new_annotation(
    self,
    id: String,
    point: AnnotationPoint,
    style: Option<&super::super::AnnotationStyle>,
    angle: Option<f64>,
    existing: &[super::super::Annotation],
    source_per_output: f64,
  ) -> super::super::Annotation {
    match self {
      Self::Arrow => super::super::arrow::model::new_arrow(id, point, point, style),
      Self::Counter => super::super::counter::model::new_counter(
        id,
        point,
        super::super::counter::next_counter_value(existing),
        style,
        angle,
      ),
      Self::Text => super::super::text::new_text(id, point, style, source_per_output),
      Self::Redact => super::super::redact::new_redact(id, point, style),
    }
  }

  /// The reveal window a clip of this kind is at: an arrow is drawn along its
  /// own path; a counter grows into place on its own quicker timing, and a
  /// text box does the same before its pointer draws out. A redaction ramps
  /// in on the counter's timing and stays whole to its clip's last frame.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn reveal_window(
    self,
    elapsed_ms: f32,
    duration_ms: f32,
    frame_ms: f32,
  ) -> AnnotationReveal {
    match self {
      Self::Arrow => super::super::reveal::reveal_window(elapsed_ms, duration_ms, frame_ms),
      Self::Counter => {
        super::super::counter::reveal::counter_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
      Self::Text => {
        super::super::text::reveal::text_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
      Self::Redact => {
        super::super::redact::reveal::redact_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
    }
  }
}
