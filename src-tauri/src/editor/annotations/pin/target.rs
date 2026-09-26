// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a pinned annotation follows, worked out from the annotation itself:
//! the user never draws a tracking area.

use std::hash::{Hash, Hasher};

use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// How far a patch around a point reaches on each side, as a share of the
/// recording's longer side: about a button's worth on a Retina screen.
const PATCH_SHARE: f64 = 0.025;
/// The least a patch reaches, in source pixels.
const MIN_PATCH: f64 = 40.0;

/// What a pin follows, in source pixels where the annotation was drawn: the
/// content in `region`, and the point whose place on the frame decides
/// whether the annotation shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PinTarget {
  pub(crate) region: [f64; 4],
  pub(crate) anchor: [f64; 2],
  /// A redaction: its points are taken from the middle of its box, and it
  /// shows for as long as any of its box is on the frame, since the part of
  /// what it hides still on the frame must stay hidden.
  pub(crate) redaction: bool,
}

impl PinTarget {
  /// The content an annotation points at: an arrow's tip, a counter's disc,
  /// the corner a text box was dropped at, and everything a redaction covers.
  /// `source` is the recording's size, which sizes the patch around a point.
  pub(crate) fn of(annotation: &Annotation, source: (u32, u32)) -> Self {
    let half = (f64::from(source.0.max(source.1)) * PATCH_SHARE).max(MIN_PATCH);
    let around = |point: AnnotationPoint| Self {
      region: [
        point.x - half,
        point.y - half,
        point.x + half,
        point.y + half,
      ],
      anchor: [point.x, point.y],
      redaction: false,
    };
    match &annotation.shape {
      // The head rides the end point.
      AnnotationShape::Arrow { end, .. } => around(*end),
      AnnotationShape::Counter { center, .. } => around(*center),
      AnnotationShape::Text { origin, .. } => around(*origin),
      AnnotationShape::Redact { start, end, .. } => {
        let region = [
          start.x.min(end.x),
          start.y.min(end.y),
          start.x.max(end.x),
          start.y.max(end.y),
        ];
        Self {
          anchor: [0.5 * (region[0] + region[2]), 0.5 * (region[1] + region[3])],
          region,
          redaction: true,
        }
      }
    }
  }

  /// Moved by `[dx, dy]`.
  pub(crate) fn shifted(&self, [dx, dy]: [f64; 2]) -> Self {
    Self {
      region: [
        self.region[0] + dx,
        self.region[1] + dy,
        self.region[2] + dx,
        self.region[3] + dy,
      ],
      anchor: [self.anchor[0] + dx, self.anchor[1] + dy],
      redaction: self.redaction,
    }
  }

  pub(crate) fn hash_into(&self, hasher: &mut impl Hasher) {
    for value in self.region.iter().chain(&self.anchor) {
      value.to_bits().hash(hasher);
    }
    self.redaction.hash(hasher);
  }
}

/// The point of `annotation` whose place decides whether it shows: the one
/// [`PinTarget::of`] follows.
pub(crate) fn anchor_of(annotation: &Annotation) -> [f64; 2] {
  PinTarget::of(annotation, (0, 0)).anchor
}
