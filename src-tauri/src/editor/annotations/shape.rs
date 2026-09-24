// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an annotation is, and the list of what each kind has to answer.
//!
//! Every per-kind branch in the editor is here, and every arm is one call
//! into that kind's own module. Adding a tool is therefore a compile error
//! until each arm below has been answered, rather than a silent fall back to
//! the arrow.

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
  /// Lines of type in a solid box. `origin` is the box's top-left corner and
  /// `pointer` the pointer drawn out of it, held against the box. The box's
  /// size follows from the text and the type size.
  Text {
    origin: AnnotationPoint,
    #[serde(default)]
    pointer: super::text::model::TextPointer,
    text: String,
  },
}

impl AnnotationShape {
  /// Which kind the shape is, which is what the native records carry.
  pub(crate) fn kind(&self) -> AnnotationKind {
    match self {
      Self::Arrow { .. } => AnnotationKind::Arrow,
      Self::Counter { .. } => AnnotationKind::Counter,
      Self::Text { .. } => AnnotationKind::Text,
    }
  }

  /// The points the shape is placed by, for the coarse bounds and finiteness
  /// tests every space-changing path runs. A counter reports its centre
  /// three times and a text box its corner: the disc, the tail and the box
  /// all reach past them, so a box over these points is smaller than the
  /// annotation. A text box's pointer is held against the box, not placed.
  pub(crate) fn points(&self) -> [AnnotationPoint; 3] {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => [*start, *control, *end],
      Self::Counter { center, .. } => [*center; 3],
      Self::Text { origin, .. } => [*origin; 3],
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
      Self::Text {
        origin, pointer, ..
      } => super::text::model::placed(*origin, pointer),
    }
  }

  /// The same shape with every point moved by `map`. What an annotation *is*
  /// does not change with the space it is drawn in, so the angle, the number
  /// and the text ride through untouched: every space an annotation travels
  /// between keeps the picture's aspect, so a direction in one is the same
  /// direction in the next.
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
      Self::Text {
        origin,
        pointer,
        text,
      } => Self::Text {
        origin: map(*origin),
        pointer: *pointer,
        text: text.clone(),
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
      Self::Counter { .. } | Self::Text { .. } => super::arrow::bend::ArrowBend::STRAIGHT,
    }
  }

  /// The three points the retained draw record carries. Each kind reads the
  /// slots its own way; [`super::native`] documents every reading.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_points(&self, style: &super::AnnotationStyle) -> [[f32; 2]; 3] {
    match self {
      Self::Arrow {
        start,
        control,
        end,
      } => super::arrow::native::draw_points(*start, *control, *end),
      Self::Counter { center, angle, .. } => super::counter::native::draw_points(*center, *angle),
      Self::Text {
        origin,
        pointer,
        text,
      } => super::text::native::draw_points(*origin, pointer, text, style),
    }
  }

  /// The `head` the retained draw record carries: which ends of an arrow
  /// have one, nothing for a counter, and a text box's alignment.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_head(&self, style: &super::AnnotationStyle) -> u32 {
    match self {
      Self::Arrow { .. } | Self::Counter { .. } => match style.head {
        super::AnnotationHead::None => 0,
        super::AnnotationHead::End => 1,
        super::AnnotationHead::Both => 2,
      },
      Self::Text { .. } => style.align.raw(),
    }
  }

  /// What the retained draw record rasterises as type: a counter's number, a
  /// text box's text, nothing for an arrow.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn draw_text(&self) -> std::borrow::Cow<'_, str> {
    match self {
      Self::Arrow { .. } => std::borrow::Cow::Borrowed(""),
      Self::Counter { value, .. } => std::borrow::Cow::Owned(value.to_string()),
      Self::Text { text, .. } => std::borrow::Cow::Borrowed(text),
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
      Self::Text {
        origin,
        pointer,
        text,
      } => super::text::handles::grips(*origin, pointer, text, style, index, source, image_width),
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
      Self::Text { origin, text, .. } => {
        super::text::snap::field_box(*origin, text, width, source_per_output)
      }
    }
  }
}

/// What a gesture asks of each kind: how a grip moves it, how a fresh one is
/// made and carried, and how it arrives over its clip.
#[path = "shape_edit.rs"]
mod edit;
