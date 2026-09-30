// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving a magnifier's grips, and pulling a fresh zoom area out.
//!
//! The zoom area's grips are a box's, and act as a spotlight's do: resizing
//! it changes what is enlarged and leaves the loupe as big as it was. The
//! loupe has two of its own: its inside, which carries it, and the grip on
//! its rim, which sizes it and so sets the zoom. Every sample is measured
//! against the magnifier the press began on.

use super::geometry::unit_grip;
use super::model::{longest, MAX_MAGNIFY_ZOOM, MIN_MAGNIFY_ZOOM, NEW_MAGNIFY_ZOOM};
use crate::editor::annotations::box_gesture;
use crate::editor::annotations::gesture::{AnnotationDragOrigin, AnnotationHandle};
use crate::editor::annotations::snap::{SnapRequest, SnapResult};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// Move one grip of a magnifier to `point`. The loupe's inside carries the
/// loupe, its rim grip sizes it, and every other grip is the zoom area's.
pub(crate) fn drag(
  annotation: &mut Annotation,
  handle: AnnotationHandle,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let AnnotationShape::Magnify {
    start: from_start,
    end: from_end,
    loupe: from_loupe,
    ..
  } = origin.shape
  else {
    return SnapResult::default();
  };
  match handle {
    AnnotationHandle::Middle => {
      if let AnnotationShape::Magnify { loupe, .. } = &mut annotation.shape {
        *loupe = AnnotationPoint {
          x: from_loupe.x + point.x - origin.point.x,
          y: from_loupe.y + point.y - origin.point.y,
        };
      }
      SnapResult::default()
    }
    AnnotationHandle::Tail => {
      // The grip sits on the rim at `unit` per unit of the loupe's size, so
      // the size that puts it nearest the pointer is the pointer's reach
      // along that direction.
      let half = [
        ((from_end.x - from_start.x).abs() / 2.0) as f32,
        ((from_end.y - from_start.y).abs() / 2.0) as f32,
      ];
      let unit = unit_grip(half).map(f64::from);
      let reach = (point.x - from_loupe.x) * unit[0] + (point.y - from_loupe.y) * unit[1];
      let length = unit[0] * unit[0] + unit[1] * unit[1];
      let longest = longest(from_start, from_end);
      if let AnnotationShape::Magnify { size, .. } = &mut annotation.shape {
        if length > 0.0 && longest > 0.0 {
          *size = (reach / length).clamp(longest * MIN_MAGNIFY_ZOOM, longest * MAX_MAGNIFY_ZOOM);
        }
      }
      SnapResult::default()
    }
    _ => {
      let result = box_gesture::drag(annotation, handle, point, origin, shift, snap);
      keep_enlarging(annotation);
      result
    }
  }
}

/// Keeps the loupe at least the least zoom's size over a zoom area that has
/// grown towards it.
fn keep_enlarging(annotation: &mut Annotation) {
  if let AnnotationShape::Magnify {
    start, end, size, ..
  } = &mut annotation.shape
  {
    *size = size.max(longest(*start, *end) * MIN_MAGNIFY_ZOOM);
  }
}

/// Pull a fresh zoom area out from where the press landed, as a spotlight's
/// box is, and set its loupe beside it at twice its size, on the first side
/// with room for it inside a picture `origin.source_size` across.
pub(crate) fn drag_new(
  annotation: &mut Annotation,
  point: AnnotationPoint,
  origin: &AnnotationDragOrigin,
  shift: bool,
  snap: Option<SnapRequest<'_>>,
) -> SnapResult {
  let result = box_gesture::drag_new(annotation, point, origin, shift, snap);
  if let AnnotationShape::Magnify {
    start,
    end,
    loupe,
    size,
  } = &mut annotation.shape
  {
    *size = longest(*start, *end) * NEW_MAGNIFY_ZOOM;
    *loupe = beside(*start, *end, NEW_MAGNIFY_ZOOM, origin.source_size);
  }
  result
}

/// Where a loupe `zoom` times the zoom area from `start` to `end` goes beside
/// it: to its right, its left, below or above, the first that fits inside a
/// picture `bounds` across, or wherever spills out the least. A gap of a
/// quarter of the zoom area keeps the two apart for the line between them.
pub(crate) fn beside(
  start: AnnotationPoint,
  end: AnnotationPoint,
  zoom: f64,
  bounds: (u32, u32),
) -> AnnotationPoint {
  let half = ((end.x - start.x).abs() / 2.0, (end.y - start.y).abs() / 2.0);
  let centre = ((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
  let reach = (half.0 * (1.5 + zoom), half.1 * (1.5 + zoom));
  let loupe = (half.0 * zoom, half.1 * zoom);
  let places = [
    (centre.0 + reach.0, centre.1),
    (centre.0 - reach.0, centre.1),
    (centre.0, centre.1 + reach.1),
    (centre.0, centre.1 - reach.1),
  ];
  let (width, height) = (f64::from(bounds.0), f64::from(bounds.1));
  let spill = |(x, y): (f64, f64)| {
    if width <= 0.0 || height <= 0.0 {
      return 0.0;
    }
    let over = |low: f64, high: f64, limit: f64| (-low).max(0.0) + (high - limit).max(0.0);
    over(x - loupe.0, x + loupe.0, width) + over(y - loupe.1, y + loupe.1, height)
  };
  let best = places
    .iter()
    .copied()
    .find(|place| spill(*place) <= 0.0)
    .unwrap_or_else(|| {
      places
        .iter()
        .copied()
        .min_by(|a, b| spill(*a).total_cmp(&spill(*b)))
        .unwrap_or(places[0])
    });
  AnnotationPoint {
    x: best.0,
    y: best.1,
  }
}
