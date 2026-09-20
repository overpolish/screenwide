// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an annotation lands while the positional modifier is held.
//!
//! The layer engines snap a rectangle's edges and centres to the canvas and
//! to the other layers, natively and once per platform. An annotation has two
//! different ideas of a target, so the rules live here instead: a counter's
//! disc aligns its edges and its centre to the canvas and to the other discs,
//! and an arrow's tip lands on an edge of a detected UI element. The
//! thresholds and the tie-break are the layer engines' - eight screen pixels,
//! nearest wins, an object beats the canvas - converted into source pixels
//! per sample, because that is the space the whole annotation model works in.
//!
//! What moves is a [`SnapSubject`]: the lines the shape offers in each axis.
//! The engine resolves one offset per axis and applies it to all of them, so
//! a shape aligns by whichever part of it is nearest a candidate without ever
//! being stretched. A tool that wants its own candidates adds them to
//! [`SnapField`]; nothing below this module knows where a candidate came from.
//!
//! An arrow's tip rides an element's whole edge rather than a handful of
//! points along it: one axis is pinned and the other still follows the hand,
//! so the arrow can be aimed anywhere down a button's side and lands on a
//! corner only where two edges are both in reach. The edges belong to the
//! detected box grown by [`element_padding`], because an arrowhead butted
//! against the thing it points at reads worse than one a hair off it. A
//! dashed or dotted glyph is detected as one small box per dash, so the
//! fragments are folded into the single element they make up before any edge
//! is offered.

use super::{Annotation, AnnotationPoint};

#[path = "snap_anchors.rs"]
mod anchors;
pub(crate) use anchors::{detect_anchors, request_anchors, AnchorBoxes, AnchorCache};

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;

/// How far a snap reaches, in screen points. The layer engines' own distance.
const THRESHOLD_POINTS: f64 = 8.0;

/// What one gesture can land on.
#[path = "snap_field.rs"]
mod field;
pub(crate) use field::{disc_radius, SnapField};

/// The edges of a detected element, which an arrow's tip rides.
#[path = "snap_edges.rs"]
mod edges;
pub(crate) use edges::{element_padding, merge_fragments, snap_edges, SnapBounds};

/// The lines one axis offers, and which of them a shape takes.
#[path = "snap_axis.rs"]
mod axis;
pub(crate) use axis::{resolve_axis, snap_axis, AxisGuide, AxisWinner};

/// Equal spacing with a pair of other annotations.
#[path = "snap_gaps.rs"]
mod gaps;
pub(crate) use gaps::{snap_gap, GapCandidate, GapSnap, GapSpan};

/// What moves, and the rectangles it aligns to.
#[path = "snap_subject.rs"]
mod subject;
pub(crate) use subject::{Axis, SnapBox, SnapOffset, SnapSubject};

/// Where a turning tail's tip can land.
#[path = "snap_tail.rs"]
mod tail;
pub(crate) use tail::snap_tail;

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
  /// The pair of equal gaps each axis landed on, when that is what it took.
  pub(crate) gap_x: Option<GapSnap>,
  pub(crate) gap_y: Option<GapSnap>,
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

/// A field and the reach one sample gets over it. Absent when the modifier is
/// not held, or when the picture's on-screen size is not known well enough to
/// turn screen points into source pixels.
#[derive(Clone, Copy)]
pub(crate) struct SnapRequest<'a> {
  pub(crate) field: &'a SnapField,
  pub(crate) threshold: f64,
}

impl SnapRequest<'_> {
  /// Where a counter lands: its disc aligned to the guides, its tail tip
  /// competing for the detected elements' edges, and the disc offered equal
  /// spacing with every pair of other annotations. Each axis resolves once
  /// and one offset carries the whole counter, so it arrives whole however
  /// the candidates fell out.
  pub(crate) fn counter(&self, disc: SnapBox, tip: AnnotationPoint) -> (SnapOffset, SnapResult) {
    let subject = SnapSubject::moved_box(disc);
    let edges = self
      .field
      .anchors
      .as_deref()
      .and_then(|anchors| snap_edges(tip, anchors.bounds(), self.threshold));
    let gaps =
      [Axis::X, Axis::Y].map(|axis| snap_gap(disc, &self.field.boxes, axis, self.threshold));
    let (x, took_x) = resolve_axis(
      snap_axis(subject.x.as_slice(), &self.field.guides_x, self.threshold),
      edges.and_then(|edge| edge.x).map(|edge| edge - tip.x),
      gaps[0].map(|gap| gap.offset),
    );
    let (y, took_y) = resolve_axis(
      snap_axis(subject.y.as_slice(), &self.field.guides_y, self.threshold),
      edges.and_then(|edge| edge.y).map(|edge| edge - tip.y),
      gaps[1].map(|gap| gap.offset),
    );
    let offset = SnapOffset { x, y };
    // The bars are measured from where the counter landed, not from where
    // the hand was, so each sits in the middle of what the boxes either side
    // of it really share.
    let moved = disc.moved(offset);
    let took_edge = |taken: Option<AxisWinner>| taken.is_some_and(AxisWinner::is_edge);
    let gap = |taken: Option<AxisWinner>, candidate: Option<GapCandidate>, axis| {
      taken
        .filter(|taken| taken.is_gap())
        .and(candidate)
        .map(|candidate| candidate.spans(moved, axis))
    };
    (
      offset,
      SnapResult {
        guide_x: took_x.and_then(AxisWinner::guide),
        guide_y: took_y.and_then(AxisWinner::guide),
        anchor: edges
          .filter(|_| took_edge(took_x) || took_edge(took_y))
          .map(|edge| SnapAnchor {
            point: offset.apply(tip),
            bounds: edge.bounds,
          }),
        gap_x: gap(took_x, gaps[0], Axis::X),
        gap_y: gap(took_y, gaps[1], Axis::Y),
      },
    )
  }

  /// The angle that lands a counter's tail tip on a detected element's edge,
  /// and where it lands. Absent when no edge is within reach of the aim.
  pub(crate) fn tail(
    &self,
    center: AnnotationPoint,
    radius: f64,
    angle: f64,
  ) -> Option<(f64, SnapAnchor)> {
    let anchors = self.field.anchors.as_deref()?;
    snap_tail(center, radius, angle, anchors.bounds(), self.threshold)
      .map(|(angle, point, bounds)| (angle, SnapAnchor { point, bounds }))
  }

  /// The nearest element edge to `tip`. A field with no detected elements - a
  /// camera pane, or a frame whose detection has not landed yet - snaps to
  /// nothing.
  pub(crate) fn anchor(&self, tip: AnnotationPoint) -> Option<SnapAnchor> {
    let anchors = self.field.anchors.as_deref()?;
    let edge = snap_edges(tip, anchors.bounds(), self.threshold)?;
    Some(SnapAnchor {
      point: AnnotationPoint {
        x: edge.x.unwrap_or(tip.x),
        y: edge.y.unwrap_or(tip.y),
      },
      bounds: edge.bounds,
    })
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
