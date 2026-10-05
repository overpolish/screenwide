// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each kind sits in the source's pixels: the points it is placed by,
//! whether it can be drawn there, and the same shape carried into another
//! space.

use super::super::{
  arrow, counter, freehand, highlight, image, magnify, outline, redact, spotlight, text,
  AnnotationPoint,
};

impl super::AnnotationShape {
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
      Self::Redact { start, end, .. }
      | Self::Shape { start, end, .. }
      | Self::Spotlight { start, end } => [*start, *end, *end],
      Self::Magnify {
        start, end, loupe, ..
      } => [*start, *end, *loupe],
      Self::Image {
        center,
        size,
        angle,
        aspect,
        ..
      } => {
        let (low, high) = image::model::bounds(*center, *size, *angle, *aspect);
        [low, high, *center]
      }
      Self::Highlight {
        start, end, bands, ..
      } => {
        let (low, high) = highlight::model::bounds(*start, *end, bands);
        [low, high, high]
      }
      Self::Draw { points, .. } => {
        let (low, high) = freehand::model::bounds(points);
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
      } => arrow::model::placed(*start, *control, *end),
      Self::Counter { center, angle, .. } => counter::model::placed(*center, *angle),
      Self::Text {
        origin, pointer, ..
      } => text::model::placed(*origin, pointer),
      Self::Redact { start, end, .. } => redact::model::placed(*start, *end),
      Self::Shape { start, end, .. } => outline::model::placed(*start, *end),
      Self::Spotlight { start, end } => spotlight::model::placed(*start, *end),
      Self::Highlight {
        start, end, bands, ..
      } => highlight::model::placed(*start, *end, bands),
      Self::Draw { points, .. } => freehand::model::placed(points),
      Self::Magnify {
        start,
        end,
        loupe,
        size,
      } => magnify::model::placed(*start, *end, *loupe, *size),
      Self::Image {
        center,
        size,
        angle,
        aspect,
        ..
      } => image::model::placed(*center, *size, *angle, *aspect),
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
      Self::Spotlight { start, end } => Self::Spotlight {
        start: map(*start),
        end: map(*end),
      },
      Self::Draw { points, smooth } => Self::Draw {
        points: points.iter().map(|point| map(*point)).collect(),
        smooth: *smooth,
      },
      Self::Magnify {
        start,
        end,
        loupe,
        size,
      } => {
        // A length rides through as far as the space it moves into
        // stretches it; every such space keeps the picture's aspect.
        let centre = map(*loupe);
        let edge = map(AnnotationPoint {
          x: loupe.x + size,
          y: loupe.y,
        });
        Self::Magnify {
          start: map(*start),
          end: map(*end),
          loupe: centre,
          size: (edge.x - centre.x).abs(),
        }
      }
      Self::Image {
        center,
        size,
        angle,
        aspect,
        flip,
        asset,
        play,
        sway,
        clock_ms,
      } => {
        // Its size rides through as the magnifier's does; the turn, the
        // mirroring, how it plays and how it sways are the same in every
        // space, the drift being a share of its size.
        let centre = map(*center);
        let edge = map(AnnotationPoint {
          x: center.x + size,
          y: center.y,
        });
        Self::Image {
          center: centre,
          size: (edge.x - centre.x).abs(),
          angle: *angle,
          aspect: *aspect,
          flip: *flip,
          asset: asset.clone(),
          play: *play,
          sway: *sway,
          clock_ms: *clock_ms,
        }
      }
    }
  }
}
