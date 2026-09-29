// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a pinned annotation follows, worked out from the annotation itself:
//! the user never draws a tracking area.

use std::hash::{Hash, Hasher};

use crate::editor::annotations::counter::silhouette::counter_tail_tip;
use crate::editor::annotations::snap::disc_radius;
use crate::editor::annotations::text::gesture::pointer_tip;
use crate::editor::annotations::text::snap::text_box;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// How far a patch around a point reaches on each side, as a share of the
/// recording's longer side: about a button's worth on a Retina screen.
const PATCH_SHARE: f64 = 0.025;
/// The least a patch reaches, in source pixels.
const MIN_PATCH: f64 = 40.0;
/// How far a tip's patch is moved ahead of it, as a share of the patch's
/// reach: half, which leaves a quarter of the patch behind the tip.
const AHEAD_SHARE: f64 = 0.5;

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
  /// The content an annotation points at: an arrow's tip, a counter's tail
  /// tip, a text box's pointer tip (or its corner when the pointer is tucked
  /// in), and everything a redaction covers. `source` is the recording's size,
  /// which sizes the patch around a point.
  ///
  /// What a tip points at lies ahead of it, so the patch around a tip is
  /// moved forward along the way it points: three quarters of it ahead, and
  /// a quarter behind to keep what the tip sits on.
  ///
  /// A counter and a text box are sized in points, so where their tips fall
  /// in the recording takes `source_per_size`, source pixels per point of
  /// their size. Where that is not known yet (zero), a counter follows its
  /// centre and a text box its corner.
  pub(crate) fn of(annotation: &Annotation, source: (u32, u32), source_per_size: f64) -> Self {
    let scale = if source_per_size.is_finite() {
      source_per_size.max(0.0)
    } else {
      0.0
    };
    let width = annotation.style.width;
    let half = (f64::from(source.0.max(source.1)) * PATCH_SHARE).max(MIN_PATCH);
    let around = |point: AnnotationPoint, from: AnnotationPoint| {
      let (x, y) = (point.x - from.x, point.y - from.y);
      let length = x.hypot(y);
      let (cx, cy) = if length > 1e-6 {
        (
          point.x + half * AHEAD_SHARE * x / length,
          point.y + half * AHEAD_SHARE * y / length,
        )
      } else {
        (point.x, point.y)
      };
      Self {
        region: [cx - half, cy - half, cx + half, cy + half],
        anchor: [point.x, point.y],
        redaction: false,
      }
    };
    match &annotation.shape {
      // The head rides the end point, arriving from the curve's control
      // point; a straight arrow's control point may sit on its end.
      AnnotationShape::Arrow {
        start,
        control,
        end,
      } => {
        let arrives = (control.x - end.x).hypot(control.y - end.y) > 1e-6;
        around(*end, if arrives { *control } else { *start })
      }
      AnnotationShape::Counter { center, angle, .. } => {
        let radius = disc_radius(width, scale);
        around(counter_tail_tip(*center, radius, *angle), *center)
      }
      AnnotationShape::Text {
        origin,
        pointer,
        text,
      } => {
        if scale <= 0.0 {
          return around(*origin, *origin);
        }
        let bounds = text_box(*origin, text, width, scale);
        let middle = AnnotationPoint {
          x: bounds.x + bounds.width / 2.0,
          y: bounds.y + bounds.height / 2.0,
        };
        match pointer_tip(bounds, width.max(0.0) * scale, pointer) {
          Some(tip) => around(tip, middle),
          None => around(*origin, *origin),
        }
      }
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
      // A highlight follows the text it covers: every band, held as one
      // region, and never resized with it.
      AnnotationShape::Highlight {
        start, end, bands, ..
      } => {
        let (low, high) = crate::editor::annotations::highlight::model::bounds(*start, *end, bands);
        Self {
          anchor: [0.5 * (low.x + high.x), 0.5 * (low.y + high.y)],
          region: [low.x, low.y, high.x, high.y],
          redaction: false,
        }
      }
      // A shape follows what it outlines as one region, and keeps its size;
      // a spotlight follows what it lights the same way.
      AnnotationShape::Shape { start, end, .. } | AnnotationShape::Spotlight { start, end } => {
        Self {
          anchor: [0.5 * (start.x + end.x), 0.5 * (start.y + end.y)],
          region: [
            start.x.min(end.x),
            start.y.min(end.y),
            start.x.max(end.x),
            start.y.max(end.y),
          ],
          redaction: false,
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

  /// With each side of the region moved out by `edges` - left, top, right,
  /// bottom - where a keyframe resized the box: the content followed from
  /// that keyframe is what the resized box covers. The anchor stays, so a
  /// movement is still measured from where the annotation was drawn.
  pub(crate) fn edged(&self, edges: Option<[f64; 4]>) -> Self {
    let Some([left, top, right, bottom]) = edges else {
      return *self;
    };
    let [x0, y0, x1, y1] = self.region;
    let (x0, x1) = (x0 - left, x1 + right);
    let (y0, y1) = (y0 - top, y1 + bottom);
    Self {
      region: [x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)],
      ..*self
    }
  }

  pub(crate) fn hash_into(&self, hasher: &mut impl Hasher) {
    for value in self.region.iter().chain(&self.anchor) {
      value.to_bits().hash(hasher);
    }
    self.redaction.hash(hasher);
  }
}

/// The point `annotation` is scaled about when its pin carries it. Only a
/// redaction changes size, and its anchor is the middle of its box, which
/// takes no canvas scale; the other kinds only move, so any point will do.
pub(crate) fn anchor_of(annotation: &Annotation) -> [f64; 2] {
  PinTarget::of(annotation, (0, 0), 0.0).anchor
}
