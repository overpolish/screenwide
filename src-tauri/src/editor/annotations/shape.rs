// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an annotation is, and the list of what each kind has to answer.
//!
//! Every per-kind branch in the editor is here, and every arm is one call
//! into that kind's own module. Adding a tool is therefore a compile error
//! until each arm below has been answered, rather than a silent fall back to
//! the arrow.

use super::reveal::AnnotationReveal;
use super::{AnnotationKind, AnnotationPoint};
use serde::{Deserialize, Serialize};

/// What an annotation is. The tag leaves room for the shapes later tools add
/// without reshaping stored documents.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AnnotationShape {
  /// A quadratic Bézier from `start` to `end`, bent by `control`.
  Arrow {
    start: AnnotationPoint,
    control: AnnotationPoint,
    end: AnnotationPoint,
  },
  /// A numbered disc with a pin's curved tail. `value` is the annotation's
  /// place in the document's counter order, which the editor keeps contiguous,
  /// and `angle` is where the tail points, in radians clockwise from east in
  /// the source's own pixel space - so a fresh counter's zero points right.
  Counter {
    center: AnnotationPoint,
    value: u32,
    angle: f64,
  },
}

impl AnnotationShape {
  /// Which kind the shape is, which is what the native records carry.
  pub(crate) fn kind(&self) -> AnnotationKind {
    match self {
      Self::Arrow { .. } => AnnotationKind::Arrow,
      Self::Counter { .. } => AnnotationKind::Counter,
    }
  }

  /// The points the shape is placed by, for the coarse bounds and finiteness
  /// tests every space-changing path runs. A counter reports its centre
  /// three times: its tail reaches past it, so a box over these points is
  /// smaller than the annotation by up to the tail's length.
  pub(crate) fn points(&self) -> [AnnotationPoint; 3] {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => [*start, *control, *end],
      Self::Counter { center, .. } => [*center; 3],
    }
  }

  /// Whether the shape is somewhere it can be drawn. A document read from
  /// disk carries whatever it was written with.
  pub(crate) fn placed(&self) -> bool {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::arrow::model::placed(*start, *control, *end),
      Self::Counter { center, angle, .. } => super::counter::model::placed(*center, *angle),
    }
  }

  /// The same shape with every point moved by `map`. What an annotation *is*
  /// does not change with the space it is drawn in, so the angle and the number
  /// ride through untouched: every space an annotation travels between keeps
  /// the picture's aspect, so a direction in one is the same direction in the
  /// next.
  pub(crate) fn mapped(&self, map: impl Fn(AnnotationPoint) -> AnnotationPoint) -> Self {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => Self::Arrow {
        start: map(*start),
        control: map(*control),
        end: map(*end),
      },
      Self::Counter {
        center,
        value,
        angle,
      } => Self::Counter {
        center: map(*center),
        value: *value,
        angle: *angle,
      },
    }
  }

  /// How the shape was bent when a drag began. A shape with no curve is never
  /// bent, and its bend is never read.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn bend(&self) -> super::arrow::bend::ArrowBend {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::arrow::bend::arrow_bend(*start, *control, *end).clamped(),
      Self::Counter { .. } => super::arrow::bend::ArrowBend::STRAIGHT,
    }
  }

  /// The three points the retained draw record carries. Each kind reads the
  /// slots its own way; [`super::native`] documents both readings.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_points(&self) -> [[f32; 2]; 3] {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::arrow::native::draw_points(*start, *control, *end),
      Self::Counter {
        center,
        value,
        angle,
      } => super::counter::native::draw_points(*center, *value, *angle),
    }
  }

  /// The grips the native chrome draws, normalised over the source image.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn grips(
    &self,
    style: &super::AnnotationStyle,
    index: u32,
    source: (u32, u32),
    image_width: f64,
  ) -> super::handles::NativeAnnotationHandles {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::arrow::handles::grips(*start, *control, *end, style, index, source, image_width),
      Self::Counter { center, angle, .. } => {
        super::counter::handles::grips(*center, *angle, style, index, source, image_width)
      }
    }
  }

  /// The rectangle this annotation offers the snap engine, in the source's
  /// pixels, or `None` for a shape nothing lines up against.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn field_box(
    &self,
    width: f64,
    source_per_output: f64,
  ) -> Option<super::snap::SnapBox> {
    match self {
      Self::Arrow { .. } => super::arrow::snap::field_box(),
      Self::Counter { center, .. } => {
        super::counter::snap::field_box(*center, width, source_per_output)
      }
    }
  }
}

impl super::Annotation {
  /// Move one grip to `point`, in source pixels. `origin` is where the drag
  /// began, which is what a whole-annotation move measures its travel
  /// against, and `shift` is whether Shift was held - which holds a
  /// counter's tail to the quarter turns.
  ///
  /// `snap` is the positional candidates this sample may land on, absent
  /// when the positional modifier is not held. A counter's disc aligns to
  /// the axis guides and an arrow's tip takes an element anchor; neither
  /// shape ever sees the other's candidates. What it landed on is reported
  /// back for the chrome to draw.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn drag_grip(
    &mut self,
    handle: super::gesture::AnnotationHandle,
    point: AnnotationPoint,
    origin: &super::gesture::AnnotationDragOrigin,
    shift: bool,
    snap: Option<super::snap::SnapRequest<'_>>,
  ) -> super::snap::SnapResult {
    match self.shape.kind() {
      AnnotationKind::Arrow => {
        super::arrow::gesture::drag(self, handle, point, origin, shift, snap)
      }
      AnnotationKind::Counter => {
        super::counter::gesture::drag(self, handle, point, origin, shift, snap)
      }
    }
  }

  /// Carry the annotation this gesture has just made to `point`: an arrow is
  /// drawn out from the press, a counter was dropped whole there.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn drag_new(
    &mut self,
    point: AnnotationPoint,
    snap: Option<super::snap::SnapRequest<'_>>,
  ) -> super::snap::SnapResult {
    match self.shape.kind() {
      AnnotationKind::Arrow => super::arrow::snap::drag_new(self, point, snap),
      AnnotationKind::Counter => super::counter::snap::drag_new(self, point, snap),
    }
  }
}

impl AnnotationKind {
  /// The annotation a fresh press of this tool makes at `point`, in `style`
  /// or in the tool's own first dress. `angle` is where a counter's tail
  /// points and `existing` the list it is numbered against.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn new_annotation(
    self,
    id: String,
    point: AnnotationPoint,
    style: Option<&super::AnnotationStyle>,
    angle: Option<f64>,
    existing: &[super::Annotation],
  ) -> super::Annotation {
    match self {
      Self::Arrow => super::arrow::model::new_arrow(id, point, point, style),
      Self::Counter => super::counter::model::new_counter(
        id,
        point,
        super::counter::next_counter_value(existing),
        style,
        angle,
      ),
    }
  }

  /// The reveal window a clip of this kind is at: an arrow is drawn along its
  /// own path, a counter grows into place on its own quicker timing.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn reveal_window(
    self,
    elapsed_ms: f32,
    duration_ms: f32,
    frame_ms: f32,
  ) -> AnnotationReveal {
    match self {
      Self::Arrow => super::reveal::reveal_window(elapsed_ms, duration_ms, frame_ms),
      Self::Counter => {
        super::counter::reveal::counter_reveal_window(elapsed_ms, duration_ms, frame_ms)
      }
    }
  }
}
