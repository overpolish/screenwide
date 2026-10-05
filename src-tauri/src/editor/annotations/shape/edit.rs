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
      AnnotationKind::Redact | AnnotationKind::Shape | AnnotationKind::Spotlight => {
        super::super::box_gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Highlight => {
        super::super::highlight::gesture::drag(self, handle, point, origin)
      }
      AnnotationKind::Draw => {
        super::super::freehand::gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Magnify => {
        super::super::magnify::gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Image => {
        super::super::image::gesture::drag(self, handle, point, origin, shift, snap)
      }
    }
  }

  /// Carry the annotation this gesture has just made to `point`: an arrow is
  /// drawn out from the press, a counter was dropped whole there, a text
  /// box is carried the way a placed one is moved, so it stays centred under
  /// the hand it was centred on, and a redaction, a shape or a magnifier's
  /// zoom area is pulled out from the press to the hand, square while
  /// `shift` is held. `origin` is where the gesture began.
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
      AnnotationKind::Redact | AnnotationKind::Shape | AnnotationKind::Spotlight => {
        super::super::box_gesture::drag_new(self, point, origin, shift, snap)
      }
      AnnotationKind::Highlight => super::super::highlight::gesture::drag_new(self, point, origin),
      AnnotationKind::Draw => {
        super::super::freehand::gesture::drag_new(self, point, origin);
        super::super::snap::SnapResult::default()
      }
      AnnotationKind::Magnify => {
        super::super::magnify::gesture::drag_new(self, point, origin, shift, snap)
      }
      AnnotationKind::Image => super::super::image::gesture::drag_new(self, point, snap),
    }
  }
}

impl AnnotationKind {
  /// The annotation a fresh press of this tool makes at `point`, in `style`
  /// or in the tool's own first dress. `fresh` is what the editor has
  /// settled on beyond the dress - where a counter's tail points, which
  /// picture an image shows - `existing` the list a counter is numbered
  /// against, and `source_per_size` what sizes a text box so it can centre on
  /// `point`, and an image. `None` for an image before any picture has been
  /// chosen: a press with nothing to show places nothing.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn new_annotation(
    self,
    id: String,
    point: AnnotationPoint,
    style: Option<&super::super::AnnotationStyle>,
    fresh: &super::super::edit::FreshAnnotation,
    existing: &[super::super::Annotation],
    source_per_size: f64,
  ) -> Option<super::super::Annotation> {
    Some(match self {
      Self::Arrow => super::super::arrow::model::new_arrow(id, point, point, style),
      Self::Counter => super::super::counter::model::new_counter(
        id,
        point,
        super::super::counter::next_counter_value(existing),
        style,
        fresh.angle,
      ),
      Self::Text => super::super::text::new_text(id, point, style, source_per_size),
      Self::Redact => super::super::redact::new_redact(id, point, style),
      Self::Highlight => {
        super::super::highlight::model::new_highlight(id, point, style, source_per_size)
      }
      Self::Shape => super::super::outline::model::new_shape(
        id,
        [point, point],
        super::super::highlight::model::fresh_seed(),
        style,
      ),
      Self::Spotlight => super::super::spotlight::model::new_spotlight(id, [point, point], style),
      Self::Draw => super::super::freehand::model::new_draw(id, point, style),
      Self::Magnify => super::super::magnify::model::new_magnify(id, point, style),
      Self::Image => super::super::image::model::new_image(
        id,
        point,
        fresh.image.as_ref()?,
        style,
        source_per_size,
      ),
    })
  }

  /// The reveal window a clip of this kind is at: an arrow is drawn along its
  /// own path and a shape round its outline; a counter grows into place on its
  /// own quicker timing, and a text box does the same before its pointer
  /// draws out. A redaction ramps in on the counter's timing and stays whole
  /// to its clip's last frame, and a spotlight fades in and out on its own.
  /// `path_ms` is the clip's pace: how long what travels along a path takes
  /// to draw in - the whole of an arrow, a highlight or a shape, a text box's
  /// pointer - or `None` for the kind's own unpaced time.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn reveal_window(
    self,
    elapsed_ms: f32,
    duration_ms: f32,
    frame_ms: f32,
    path_ms: Option<f32>,
  ) -> AnnotationReveal {
    let path_ms = self.path_ms(path_ms);
    match self {
      // A highlight is drawn along its bands the way an arrow is along its
      // path, in reading order, and a shape round its outline; all three
      // leave the same way.
      Self::Arrow | Self::Highlight | Self::Shape | Self::Draw => {
        super::super::reveal::reveal_window(elapsed_ms, duration_ms, frame_ms, path_ms)
      }
      Self::Counter => {
        super::super::counter::reveal::counter_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
      // An image pops in and out the way a counter does.
      Self::Image => {
        super::super::counter::reveal::counter_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
      Self::Text => {
        super::super::text::reveal::text_reveal_window(elapsed_ms, duration_ms, frame_ms, path_ms)
      }
      Self::Redact => {
        super::super::redact::reveal::redact_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
      // Joins to the spotlights either side are the clip list's to find;
      // `spotlight::handoff` works them out and asks for its reveal itself.
      Self::Spotlight => super::super::spotlight::reveal::spotlight_reveal_window(
        elapsed_ms,
        duration_ms,
        Default::default(),
        1.0,
      ),
      Self::Magnify => {
        super::super::magnify::reveal::magnify_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
    }
  }

  /// The pace a clip of this kind draws its path at: its own, where it names
  /// one that can be drawn, and otherwise the kind's unpaced time. Zero for a
  /// kind that draws no path.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  fn path_ms(self, path_ms: Option<f32>) -> f32 {
    let own = path_ms.filter(|ms| ms.is_finite() && *ms > 0.0);
    match self {
      Self::Arrow | Self::Highlight | Self::Shape | Self::Draw => {
        own.unwrap_or(super::super::reveal::REVEAL_DRAW_IN_MS)
      }
      Self::Text => own.unwrap_or(super::super::text::reveal::POINTER_IN_MS),
      Self::Counter | Self::Redact | Self::Spotlight | Self::Magnify | Self::Image => 0.0,
    }
  }

  /// How long the kind takes to arrive and leave again at its clip's pace:
  /// the least a pinned annotation's clip must have left for it to come back
  /// onto the frame.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn reveal_span_ms(self, path_ms: Option<f32>) -> f32 {
    use super::super::counter::reveal::{COUNTER_REVEAL_IN_MS, COUNTER_REVEAL_OUT_MS};
    use super::super::reveal::REVEAL_OUT_SHARE;
    let path_ms = self.path_ms(path_ms);
    match self {
      Self::Arrow | Self::Highlight | Self::Shape | Self::Draw => {
        path_ms * (1.0 + REVEAL_OUT_SHARE)
      }
      Self::Counter | Self::Image => COUNTER_REVEAL_IN_MS + COUNTER_REVEAL_OUT_MS,
      Self::Text => super::super::text::reveal::text_reveal_span_ms(path_ms),
      // A redaction ramps in and never leaves.
      Self::Redact => COUNTER_REVEAL_IN_MS,
      Self::Spotlight => {
        use super::super::spotlight::reveal::{SPOTLIGHT_FADE_IN_MS, SPOTLIGHT_FADE_OUT_MS};
        SPOTLIGHT_FADE_IN_MS + SPOTLIGHT_FADE_OUT_MS
      }
      Self::Magnify => {
        use super::super::magnify::reveal::{MAGNIFY_REVEAL_IN_MS, MAGNIFY_REVEAL_OUT_MS};
        MAGNIFY_REVEAL_IN_MS + MAGNIFY_REVEAL_OUT_MS
      }
    }
  }
}
