// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a redaction's dragged sides land, and what one offers the snap
//! engine.
//!
//! A box carried whole aligns the way a text box does, through
//! [`SnapRequest::boxed`]. A side being dragged is a single line instead: it
//! takes the nearest guide or the nearest edge of a detected element, so a
//! box can be pulled exactly onto the field it hides.

use super::model::redact_box;
use crate::editor::annotations::snap::{
  resolve_axis, snap_axis, snap_edges, AxisWinner, SnapAnchor, SnapBox, SnapRequest, SnapResult,
};
use crate::editor::annotations::AnnotationPoint;

/// A redaction aligns by its box: its edges and its centre.
pub(crate) fn field_box(start: AnnotationPoint, end: AnnotationPoint) -> Option<SnapBox> {
  Some(redact_box(start, end))
}

/// Where a dragged corner or side lands. `moves_x` and `moves_y` say which
/// axes the grip moves; the other axis is left exactly where the hand is.
pub(crate) fn snap_sides(
  request: SnapRequest<'_>,
  point: AnnotationPoint,
  moves_x: bool,
  moves_y: bool,
) -> (AnnotationPoint, SnapResult) {
  let edge = request
    .field
    .anchors
    .as_deref()
    .and_then(|anchors| snap_edges(point, anchors.bounds(), request.threshold));
  let axis = |moves: bool, at: f64, guides, edge_at: Option<f64>| {
    if !moves {
      return (0.0, None);
    }
    resolve_axis(
      snap_axis(&[at], guides, request.threshold),
      edge_at.map(|line| line - at),
      None,
    )
  };
  let (x, took_x) = axis(
    moves_x,
    point.x,
    &request.field.guides_x,
    edge.and_then(|edge| edge.x),
  );
  let (y, took_y) = axis(
    moves_y,
    point.y,
    &request.field.guides_y,
    edge.and_then(|edge| edge.y),
  );
  let landed = AnnotationPoint {
    x: point.x + x,
    y: point.y + y,
  };
  let took_edge =
    took_x.is_some_and(AxisWinner::is_edge) || took_y.is_some_and(AxisWinner::is_edge);
  (
    landed,
    SnapResult {
      guide_x: took_x.and_then(AxisWinner::guide),
      guide_y: took_y.and_then(AxisWinner::guide),
      anchor: edge.filter(|_| took_edge).map(|edge| SnapAnchor {
        point: landed,
        bounds: edge.bounds,
      }),
      gap_x: None,
      gap_y: None,
    },
  )
}
