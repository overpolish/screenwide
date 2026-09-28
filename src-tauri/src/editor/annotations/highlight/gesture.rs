// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What moving a highlight's grips does to it.
//!
//! A highlight has the selection's two ends for grips and its bands for a
//! body. Moving an end selects again from the picture the gesture holds, so the
//! highlight follows the hand from line to line exactly as it did while it was
//! drawn out. Without a picture - a recording's frame that has not decoded yet
//! - the lines it already covers are kept and only its ends slide along them.
//! The body carries the whole highlight, bands and all.
//!
//! A highlight laid by hand is a box instead, and reads no picture at all: an
//! end is a corner, moving it lays the box's strokes again, and what it covers
//! is tinted rather than recoloured, since a box is for what the fitting
//! cannot read as text.

use super::detect::select;
use super::manual::{corners, edge_for_grip, grip, strokes};
use super::model::{band_height, HighlightTone};
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::SnapResult;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Draw a fresh highlight out from where the press landed to the hand.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
) -> SnapResult {
  if let AnnotationShape::Highlight { end, .. } = &mut annotation.shape {
    *end = point;
  }
  reselect(annotation, origin);
  SnapResult::default()
}

/// Move one grip of a highlight to `point`.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
) -> SnapResult {
  match handle {
    AnnotationHandle::Start | AnnotationHandle::End => {
      let is_start = handle == AnnotationHandle::Start;
      let manual = annotation.style.manual;
      let pressed = match &origin.shape {
        AnnotationShape::Highlight {
          start, end, bands, ..
        } => Some((
          corners(*start, *end),
          bands
            .first()
            .map(|band| band.bottom - band.top)
            .filter(|tall| *tall > 0.0)
            .unwrap_or_else(|| band_height(&annotation.style, origin.source_per_size)),
        )),
        _ => None,
      };
      // A box's grip is one of its corners as it was at the press, and the
      // opposite corner stays put, whichever way the box was first drawn. The
      // grip sits half way down a stroke rather than on the corner, so the
      // corner goes wherever keeps the grip under the hand.
      let travel = (point.x - origin.point.x, point.y - origin.point.y);
      if let AnnotationShape::Highlight { start, end, .. } = &mut annotation.shape {
        match pressed.filter(|_| manual) {
          Some(((top_left, bottom_right), marker)) if is_start => {
            let at = grip(top_left.y, bottom_right.y, marker, true) + travel.1;
            *start = AnnotationPoint {
              x: top_left.x + travel.0,
              y: edge_for_grip(bottom_right.y, at, marker, true),
            };
            *end = bottom_right;
          }
          Some(((top_left, bottom_right), marker)) => {
            let at = grip(top_left.y, bottom_right.y, marker, false) + travel.1;
            *start = top_left;
            *end = AnnotationPoint {
              x: bottom_right.x + travel.0,
              y: edge_for_grip(top_left.y, at, marker, false),
            };
          }
          None if is_start => *start = point,
          None => *end = point,
        }
      }
      if manual || origin.picture.is_some() {
        reselect(annotation, origin);
      } else {
        slide(annotation, point, is_start);
      }
    }
    AnnotationHandle::Body | AnnotationHandle::Middle => carry(annotation, point, origin),
    AnnotationHandle::Tail | AnnotationHandle::Edges(_) | AnnotationHandle::Radius => {}
  }
  SnapResult::default()
}

/// Select again between the highlight's ends, from the gesture's picture, or
/// lay a box drawn by hand again.
fn reselect(annotation: &mut Annotation, origin: &AnnotationDragOrigin) {
  let height = band_height(&annotation.style, origin.source_per_size);
  let manual = annotation.style.manual;
  let AnnotationShape::Highlight {
    start,
    end,
    bands,
    tone,
    ..
  } = &mut annotation.shape
  else {
    return;
  };
  if manual {
    // The marker is the one the strokes were laid in: a fresh highlight's
    // pressed band is one marker tall, and every stroke after it is too.
    let marker = bands
      .first()
      .map(|band| band.bottom - band.top)
      .filter(|tall| *tall > 0.0)
      .unwrap_or(height);
    *bands = strokes(*start, *end, marker);
    *tone = HighlightTone::UNREAD;
    return;
  }
  let pixels = origin
    .picture
    .as_deref()
    .map(|picture| (picture.pixels(), picture.scale()));
  let selection = select(pixels, *start, *end, height);
  *bands = selection.bands;
  *tone = selection.tone;
}

/// Slide the first band's start or the last band's end to `point`, keeping
/// every line it covers. A band is never turned inside out.
fn slide(annotation: &mut Annotation, point: AnnotationPoint, is_start: bool) {
  let AnnotationShape::Highlight { bands, .. } = &mut annotation.shape else {
    return;
  };
  let single = bands.len() == 1;
  if is_start {
    if let Some(first) = bands.first_mut() {
      first.left = if single {
        point.x.min(first.right)
      } else {
        point.x
      };
      first.right = first.right.max(first.left);
    }
  } else if let Some(last) = bands.last_mut() {
    last.right = point.x.max(last.left);
  }
}

/// Carry the whole highlight by how far the hand has moved since the press,
/// measured against the highlight the press began on.
fn carry(annotation: &mut Annotation, point: AnnotationPoint, origin: &AnnotationDragOrigin) {
  if !matches!(origin.shape, AnnotationShape::Highlight { .. }) {
    return;
  }
  let dx = point.x - origin.point.x;
  let dy = point.y - origin.point.y;
  annotation.shape = origin.shape.mapped(|at| AnnotationPoint {
    x: at.x + dx,
    y: at.y + dy,
  });
}
