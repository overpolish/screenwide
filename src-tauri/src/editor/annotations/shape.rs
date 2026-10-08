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
use ts_rs::TS;

/// What an annotation is. The tag leaves room for the shapes later tools add
/// without reshaping stored documents.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
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
  /// A box left bright while everything around it dims. `start` is its
  /// top-left corner and `end` its bottom-right, in source pixels; the style
  /// rounds its corners, fades its edge and says whether what is outside it
  /// is blurred too.
  Spotlight {
    start: AnnotationPoint,
    end: AnnotationPoint,
  },
  /// A line drawn freehand: the points the hand passed through, in source
  /// pixels, thinned as it was drawn. `smooth` fits the line loosely enough
  /// that a wobbly curve comes out clean.
  Draw {
    points: Vec<AnnotationPoint>,
    #[serde(default)]
    smooth: bool,
  },
  /// A loupe showing a zoom area enlarged. `start` is the zoom area's
  /// top-left corner and `end` its bottom-right, in source pixels; `loupe` is
  /// the loupe's centre and `size` its longer side, the zoom area's shape
  /// scaled up to that.
  Magnify {
    start: AnnotationPoint,
    end: AnnotationPoint,
    loupe: AnnotationPoint,
    size: f64,
  },
  /// A picture of your own laid over the source.
  /// `center` is its middle and `size` its longer side, in source pixels;
  /// `aspect` is the picture's width over its height, `angle` how far it is
  /// turned clockwise, in radians, and `flip` whether it is mirrored across
  /// its upright axis. `asset` names the picture in the image library,
  /// `play` how it plays where the picture moves, and `sway` the seed of the
  /// slight turn and drift a recording gives it, absent where it stands still.
  Image {
    center: AnnotationPoint,
    size: f64,
    angle: f64,
    aspect: f64,
    #[serde(default)]
    flip: bool,
    asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    play: Option<super::image::ImagePlay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    sway: Option<u32>,
    /// How far into its clip the frame being drawn is, in milliseconds,
    /// where it plays or sways; absent on a screenshot and outside a clip.
    /// Derived every frame and never stored, as an annotation's reveal is.
    #[serde(skip)]
    clock_ms: Option<f64>,
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
      Self::Spotlight { .. } => AnnotationKind::Spotlight,
      Self::Draw { .. } => AnnotationKind::Draw,
      Self::Magnify { .. } => AnnotationKind::Magnify,
      Self::Image { .. } => AnnotationKind::Image,
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
      | Self::Shape { .. }
      | Self::Spotlight { .. }
      | Self::Draw { .. }
      | Self::Magnify { .. }
      | Self::Image { .. } => super::arrow::bend::ArrowBend::STRAIGHT,
    }
  }
}

/// What the native records and the chrome ask of each kind.
mod draw;

/// What a gesture asks of each kind: how a grip moves it, how a fresh one is
/// made and carried, and how it arrives over its clip.
mod edit;

/// Where each kind sits in the source's pixels, and the same shape carried
/// into another space.
mod space;
