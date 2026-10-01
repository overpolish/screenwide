// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a highlight, and what it is made of.
//!
//! `start` and `end` are where the selection was pressed and let go, in source
//! pixels; the bands are what it covers, one per line in reading order. The
//! style's width is the band's height, in points, where the selection found
//! no text to fit.

use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationShape, AnnotationStyle};
use serde::{Deserialize, Serialize};

/// One line's band, in source pixels. `left` is never past `right` nor `top`
/// past `bottom`: every edit writes a band back that way round.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct HighlightBand {
  pub left: f64,
  pub top: f64,
  pub right: f64,
  pub bottom: f64,
}

impl HighlightBand {
  /// The band between two corners, whichever way round they are given.
  pub(crate) fn between(a: AnnotationPoint, b: AnnotationPoint) -> Self {
    Self {
      left: a.x.min(b.x),
      top: a.y.min(b.y),
      right: a.x.max(b.x),
      bottom: a.y.max(b.y),
    }
  }

  /// The same band with its corners moved by `map`.
  pub(crate) fn mapped(self, map: impl Fn(AnnotationPoint) -> AnnotationPoint) -> Self {
    Self::between(
      map(AnnotationPoint {
        x: self.left,
        y: self.top,
      }),
      map(AnnotationPoint {
        x: self.right,
        y: self.bottom,
      }),
    )
  }

  pub(crate) fn finite(self) -> bool {
    [self.left, self.top, self.right, self.bottom]
      .iter()
      .all(|value| value.is_finite())
  }
}

/// The page under a highlight: how bright its surface is, and how bright the
/// ink printed on it, each from 0 to 1 as the compositor weighs a colour's
/// luminance. The recolouring maps the surface to the highlight's colour and
/// the ink to a colour that reads on it, so light text on a dark page and dark
/// text on a light one come out the same way.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct HighlightTone {
  pub surface: f64,
  pub ink: f64,
}

impl HighlightTone {
  /// No page was read under the highlight - a photo, a gradient, a picture
  /// with no colour most of it is, or a box laid by hand where there was no
  /// picture to read. Its surface and ink are the same, which a page that was
  /// read never has, and the compositor tints what is there rather than
  /// recolouring it. A highlight whose style tints is drawn with this tone
  /// whatever it read.
  pub(crate) const UNREAD: Self = Self {
    surface: 0.5,
    ink: 0.5,
  };
}

impl Default for HighlightTone {
  /// Dark ink on a light page, which is what a highlight made where nothing
  /// could be read assumes.
  fn default() -> Self {
    Self {
      surface: 1.0,
      ink: 0.0,
    }
  }
}

/// The marker every highlight is drawn with, in points: the stroke a box laid
/// by hand is covered in, and the band drawn where no text is found. The twin
/// of `DEFAULT_ANNOTATION_HIGHLIGHT_SIZE` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_HIGHLIGHT_WIDTH: f64 = 12.0;

/// The dress a fresh highlight is drawn in before anything has been chosen:
/// the palette's yellow, a highlighter's own colour.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_highlight_style() -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: 0.0,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: 0.0,
    width: NEW_HIGHLIGHT_WIDTH,
  }
}

/// A highlight pressed at `point` and not yet drawn out: one band there, as
/// tall as the style's width, which the drag that follows selects from.
/// `source_per_size` turns that width into source pixels; zero where it is
/// unknown.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_highlight(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
  source_per_size: f64,
) -> Annotation {
  let style = style.cloned().unwrap_or_else(default_highlight_style);
  let half = band_height(&style, source_per_size) * 0.5;
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Highlight {
      start: point,
      end: point,
      bands: vec![HighlightBand {
        left: point.x,
        top: point.y - half,
        right: point.x,
        bottom: point.y + half,
      }],
      tone: HighlightTone::default(),
      seed: fresh_seed(),
    },
    style,
  }
}

/// How tall a band is where no text is found, in source pixels: the style's
/// width, which is in points.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn band_height(style: &AnnotationStyle, source_per_size: f64) -> f64 {
  let scale = if source_per_size.is_finite() && source_per_size > 0.0 {
    source_per_size
  } else {
    1.0
  };
  (style.width.max(1.0) * scale).max(1.0)
}

/// A seed for the hand-drawn stroke's wobble. Nothing is hidden by it, so any
/// generator will do; the system's is the one already at hand.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn fresh_seed() -> u32 {
  getrandom::u32().unwrap_or(0x2545_f491)
}

/// Whether a highlight is somewhere it can be drawn.
pub(crate) fn placed(
  start: AnnotationPoint,
  end: AnnotationPoint,
  bands: &[HighlightBand],
) -> bool {
  start.x.is_finite()
    && start.y.is_finite()
    && end.x.is_finite()
    && end.y.is_finite()
    && bands.iter().all(|band| band.finite())
}

/// The box every band lies in, as its two corners, or the selection's own
/// ends where there is no band.
pub(crate) fn bounds(
  start: AnnotationPoint,
  end: AnnotationPoint,
  bands: &[HighlightBand],
) -> (AnnotationPoint, AnnotationPoint) {
  let Some(first) = bands.first() else {
    let band = HighlightBand::between(start, end);
    return (
      AnnotationPoint {
        x: band.left,
        y: band.top,
      },
      AnnotationPoint {
        x: band.right,
        y: band.bottom,
      },
    );
  };
  let joined = bands.iter().fold(*first, |joined, band| HighlightBand {
    left: joined.left.min(band.left),
    top: joined.top.min(band.top),
    right: joined.right.max(band.right),
    bottom: joined.bottom.max(band.bottom),
  });
  (
    AnnotationPoint {
      x: joined.left,
      y: joined.top,
    },
    AnnotationPoint {
      x: joined.right,
      y: joined.bottom,
    },
  )
}
