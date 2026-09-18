// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an annotation lands while the positional modifier is held.
//!
//! The layer engines snap a rectangle's edges and centres to the canvas and to
//! the other layers, natively and once per platform. An annotation is a point
//! rather than a rectangle and has two different ideas of a target, so the
//! rules live here instead: a counter's centre lands on an axis guide, and an
//! arrow's tip lands on an edge of a detected UI element. The thresholds and
//! the tie-break are the layer engines' - eight screen pixels, nearest wins,
//! an object beats the canvas - converted into source pixels per sample,
//! because that is the space the whole annotation model works in.
//!
//! An arrow's tip rides an element's whole edge rather than a handful of
//! points along it: one axis is pinned and the other still follows the hand,
//! so the arrow can be aimed anywhere down a button's side and lands on a
//! corner only where two edges are both in reach. The edges belong to the
//! detected box grown by [`ELEMENT_PADDING`], because an arrowhead butted
//! against the thing it points at reads worse than one a hair off it. A dashed
//! or dotted glyph is detected as one small box per dash, so the fragments are
//! folded into the single element they make up before any edge is offered.
//!
//! A tool that wants its own candidates adds them to [`SnapField`]; nothing
//! below this module knows where a candidate came from.

use super::{Annotation, AnnotationPoint, AnnotationShape};
use std::sync::Arc;

#[path = "snap_anchors.rs"]
mod anchors;
pub(crate) use anchors::{detect_anchors, request_anchors, AnchorBoxes, AnchorCache};

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;

/// How far a snap reaches, in screen points. The layer engines' own distance.
const THRESHOLD_POINTS: f64 = 8.0;

/// How far the canvas's inset guides sit from each edge, as a share of the
/// source's shorter side. The layer engines' own inset.
const CANVAS_INSET: f64 = 0.02;

/// The edges of a detected element, which an arrow's tip rides.
#[path = "snap_edges.rs"]
mod edges;
pub(crate) use edges::{element_padding, merge_fragments, snap_edges, SnapBounds};
/// One line a position can land on, in source pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AxisGuide {
  pub(crate) position: f64,
  /// Another annotation's line rather than one of the canvas's own, which
  /// wins ties and draws in the object colour.
  pub(crate) object: bool,
}

/// Where an arrow's tip landed, and the padded element whose edge it rode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SnapAnchor {
  pub(crate) point: AnnotationPoint,
  pub(crate) bounds: SnapBounds,
}

/// What one sample snapped to. Empty when it snapped to nothing, which is what
/// puts the guides away.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SnapResult {
  pub(crate) guide_x: Option<AxisGuide>,
  pub(crate) guide_y: Option<AxisGuide>,
  pub(crate) anchor: Option<SnapAnchor>,
}

impl SnapResult {
  /// Where an arrow's tip lands: the nearest element anchor when one is in
  /// reach, and the pointer itself otherwise. The anchor is recorded here as
  /// it is resolved, so the chrome can never disagree with the geometry.
  pub(crate) fn tip(
    &mut self,
    point: AnnotationPoint,
    snap: Option<SnapRequest<'_>>,
  ) -> AnnotationPoint {
    match snap.and_then(|request| request.anchor(point)) {
      Some(anchor) => {
        self.anchor = Some(anchor);
        anchor.point
      }
      None => point,
    }
  }
}

/// Which modifiers a sample was taken with. Both Shift and the positional
/// modifier (Cmd on macOS, Ctrl on Windows) are read per sample rather than
/// latched at the press, so either can be taken and let go part way through
/// a drag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SnapModifiers {
  /// Holds a counter's tail to the eighth turns.
  pub(crate) shift: bool,
  /// Snaps the position to the candidate field.
  pub(crate) position: bool,
}

impl SnapModifiers {
  const SHIFT: u32 = 1;
  const POSITION: u32 = 1 << 1;

  pub(crate) fn from_bits(bits: u32) -> Self {
    Self {
      shift: bits & Self::SHIFT != 0,
      position: bits & Self::POSITION != 0,
    }
  }
}

/// Everything one gesture can snap to, built once when it begins.
///
/// The annotation being edited is excluded when the field is built: a counter
/// that could see its own centre would never leave the place it started.
pub(crate) struct SnapField {
  pub(crate) guides_x: Vec<AxisGuide>,
  pub(crate) guides_y: Vec<AxisGuide>,
  /// The frame's detected elements, which arrive on a blocking thread and may
  /// land part way through the gesture.
  pub(crate) anchors: Option<Arc<AnchorBoxes>>,
}

impl SnapField {
  /// The canvas's own lines and every other counter's centre, in the source's
  /// pixels. `edited` is the annotation the gesture holds.
  pub(crate) fn new(source: (u32, u32), annotations: &[Annotation], edited: &str) -> Self {
    let width = f64::from(source.0.max(1));
    let height = f64::from(source.1.max(1));
    let inset = width.min(height) * CANVAS_INSET;
    let canvas = |extent: f64| {
      [inset, extent / 2.0, extent - inset].map(|position| AxisGuide {
        position,
        object: false,
      })
    };
    let mut field = Self {
      guides_x: canvas(width).to_vec(),
      guides_y: canvas(height).to_vec(),
      anchors: None,
    };
    for annotation in annotations {
      let AnnotationShape::Counter { center, .. } = annotation.shape else {
        continue;
      };
      if annotation.id == edited {
        continue;
      }
      field.guides_x.push(AxisGuide {
        position: center.x,
        object: true,
      });
      field.guides_y.push(AxisGuide {
        position: center.y,
        object: true,
      });
    }
    field
  }
}

/// A field and the reach one sample gets over it. Absent when the modifier is
/// not held, or when the picture's on-screen size is not known well enough to
/// turn screen points into source pixels.
#[derive(Clone, Copy)]
pub(crate) struct SnapRequest<'a> {
  pub(crate) field: &'a SnapField,
  pub(crate) threshold: f64,
}

impl SnapRequest<'_> {
  /// The nearest axis guide in each axis, and `centre` moved onto them.
  pub(crate) fn axes(&self, centre: AnnotationPoint) -> (AnnotationPoint, SnapResult) {
    let guide_x = snap_axis(centre.x, &self.field.guides_x, self.threshold);
    let guide_y = snap_axis(centre.y, &self.field.guides_y, self.threshold);
    (
      AnnotationPoint {
        x: guide_x.map_or(centre.x, |guide| guide.position),
        y: guide_y.map_or(centre.y, |guide| guide.position),
      },
      SnapResult {
        guide_x,
        guide_y,
        anchor: None,
      },
    )
  }

  /// The nearest element edge to `tip`. A field with no detected elements - a
  /// camera pane, or a frame whose detection has not landed yet - snaps to
  /// nothing.
  pub(crate) fn anchor(&self, tip: AnnotationPoint) -> Option<SnapAnchor> {
    let anchors = self.field.anchors.as_deref()?;
    snap_edges(tip, anchors.bounds(), self.threshold)
      .map(|(point, bounds)| SnapAnchor { point, bounds })
  }
}

/// Eight screen points in the source's pixels. `image_points` is how wide the
/// layer's picture is drawn on screen, which is the only measure the native
/// side reports; the source's own width - not the output canvas's - is what
/// the annotation model measures in. `None` leaves the sample unsnapped.
pub(crate) fn threshold_source_px(source_width: u32, image_points: f64) -> Option<f64> {
  (image_points.is_finite() && image_points > 0.0)
    .then(|| THRESHOLD_POINTS * f64::from(source_width.max(1)) / image_points)
}

/// The nearest guide to `position` within `threshold`. An object candidate
/// beats a canvas one at the same distance, exactly as the layer engines
/// resolve a tie.
pub(crate) fn snap_axis(position: f64, guides: &[AxisGuide], threshold: f64) -> Option<AxisGuide> {
  let mut best: Option<(f64, AxisGuide)> = None;
  for guide in guides {
    let distance = (guide.position - position).abs();
    if distance > threshold {
      continue;
    }
    let wins = match best {
      None => true,
      Some((chosen_distance, chosen)) => {
        distance < chosen_distance
          || (distance == chosen_distance && guide.object && !chosen.object)
      }
    };
    if wins {
      best = Some((distance, *guide));
    }
  }
  best.map(|(_, guide)| guide)
}
