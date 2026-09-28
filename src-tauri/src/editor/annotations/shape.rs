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
  /// A box that hides what is under it. `start` is its top-left corner and
  /// `end` its bottom-right, in source pixels; `seed` generates a pixelated
  /// box's blocks.
  Redact {
    start: AnnotationPoint,
    end: AnnotationPoint,
    seed: u32,
  },
  /// A marker over lines of text. `start` and `end` are where the selection
  /// was pressed and let go, in source pixels; `bands` what it covers, one
  /// per line in reading order; `tone` the page it was read from; `seed` its
  /// hand-drawn stroke's wobble.
  Highlight {
    start: AnnotationPoint,
    end: AnnotationPoint,
    #[serde(default)]
    bands: Vec<super::highlight::model::HighlightBand>,
    #[serde(default)]
    tone: super::highlight::model::HighlightTone,
    #[serde(default)]
    seed: u32,
  },
  /// An outline round a box, drawn with a round pen. `start` is the box's
  /// top-left corner and `end` its bottom-right, in source pixels; its
  /// corners are rounded by the style's radius, so a square rounded all the
  /// way is a circle. `seed` is its hand-drawn stroke's wobble and where that
  /// stroke begins and ends.
  Shape {
    start: AnnotationPoint,
    end: AnnotationPoint,
    seed: u32,
  },
}

impl AnnotationShape {
  /// Which kind the shape is, which is what the native records carry.
  pub(crate) fn kind(&self) -> AnnotationKind {
    match self {
      Self::Arrow { .. } => AnnotationKind::Arrow,
      Self::Counter { .. } => AnnotationKind::Counter,
      Self::Text { .. } => AnnotationKind::Text,
      Self::Redact { .. } => AnnotationKind::Redact,
      Self::Highlight { .. } => AnnotationKind::Highlight,
      Self::Shape { .. } => AnnotationKind::Shape,
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
      Self::Redact { start, end, .. } | Self::Shape { start, end, .. } => [*start, *end, *end],
      Self::Highlight {
        start, end, bands, ..
      } => {
        let (low, high) = super::highlight::model::bounds(*start, *end, bands);
        [low, high, high]
      }
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
      Self::Redact { start, end, .. } => super::redact::model::placed(*start, *end),
      Self::Shape { start, end, .. } => super::outline::model::placed(*start, *end),
      Self::Highlight {
        start, end, bands, ..
      } => super::highlight::model::placed(*start, *end, bands),
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
      Self::Redact { start, end, seed } => Self::Redact {
        start: map(*start),
        end: map(*end),
        seed: *seed,
      },
      Self::Highlight {
        start,
        end,
        bands,
        tone,
        seed,
      } => Self::Highlight {
        start: map(*start),
        end: map(*end),
        bands: bands.iter().map(|band| band.mapped(&map)).collect(),
        tone: *tone,
        seed: *seed,
      },
      Self::Shape { start, end, seed } => Self::Shape {
        start: map(*start),
        end: map(*end),
        seed: *seed,
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
      Self::Counter { .. }
      | Self::Text { .. }
      | Self::Redact { .. }
      | Self::Highlight { .. }
      | Self::Shape { .. } => super::arrow::bend::ArrowBend::STRAIGHT,
    }
  }
}

/// What the native records and the chrome ask of each kind.
#[path = "shape_draw.rs"]
mod draw;

/// What a gesture asks of each kind: how a grip moves it, how a fresh one is
/// made and carried, and how it arrives over its clip.
#[path = "shape_edit.rs"]
mod edit;
