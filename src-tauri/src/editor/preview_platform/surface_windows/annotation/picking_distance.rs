// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How far a press is from each kind of annotation, in display points, from
//! the grips it was published with.

use super::picking::{display_point, highlight_flow, text_geometry};
use super::*;
use crate::editor::annotations::arrow::distance::prepared_arrow_distance;
use crate::editor::annotations::arrow::geometry::prepare_arrow;
use crate::editor::annotations::counter::silhouette::counter_distance;
use crate::editor::annotations::freehand::geometry::{freehand_body_distance, freehand_distance};
use crate::editor::annotations::highlight::geometry::flow_distance;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::text_distance;
use crate::editor::annotations::AnnotationPoint;

/// How far a point is from one arrow's drawn shape, in display points: its
/// shaft, and the heads on it. Zero anywhere the arrow is actually painted,
/// because the tolerance is measured from the stroke's edge rather than its
/// centreline. A press on a head is a press on the arrow - it is the part of it
/// the hand aims at. An annotation half-way through drawing itself in is still
/// picked by the whole of what it will be.
pub(super) fn arrow_distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let start = display_point(image, item.start_x, item.start_y);
  match item.shape_kind() {
    AnnotationKind::Counter => {
      return counter_distance(
        point,
        AnnotationPoint {
          x: start.0,
          y: start.1,
        },
        item.width * image.width / 2.0,
        item.start_head,
      ) as f32;
    }
    AnnotationKind::Text => {
      return text_distance(
        [point.0 as f32, point.1 as f32],
        &text_geometry(image, item),
      );
    }
    AnnotationKind::Redact | AnnotationKind::Spotlight => {
      return super::redact_chrome::distance(image, item, point)
    }
    AnnotationKind::Shape => {
      return super::redact_chrome::shape_stroke_distance(image, item, point)
    }
    AnnotationKind::Highlight => {
      return flow_distance(
        [point.0 as f32, point.1 as f32],
        &highlight_flow(image, item),
      );
    }
    // A stroke's line is in the published paths, which `draw_distance`
    // reads; its grips alone cannot place it.
    AnnotationKind::Draw => return f32::INFINITY,
    AnnotationKind::Arrow => {}
  }
  let middle = display_point(image, item.middle_x, item.middle_y);
  let end = display_point(image, item.end_x, item.end_y);
  // The control point behind a reported middle handle, so the shaft can be
  // sampled without solving the curve again.
  let a = [start.0 as f32, start.1 as f32];
  let b = [
    (2.0 * middle.0 - (start.0 + end.0) / 2.0) as f32,
    (2.0 * middle.1 - (start.1 + end.1) / 2.0) as f32,
  ];
  let c = [end.0 as f32, end.1 as f32];
  let probe = [point.0 as f32, point.1 as f32];
  // The stroke rides on the annotation rather than being read back off its
  // heads, so a headless arrow is picked over the width it shows too.
  let width = (item.width * image.width) as f32;
  let heads = if item.start_head > 0.0 {
    2
  } else if item.end_head > 0.0 {
    1
  } else {
    0
  };
  let geometry = prepare_arrow(a, b, c, width, heads, AnnotationReveal::WHOLE);
  prepared_arrow_distance(probe, &geometry)
}

/// How far `point` is from a stroke's drawn line, in display points, from
/// the fitted line its grips point into; with `body`, its box picks it too.
/// The twin of `annotation_draw_distance`.
pub(super) fn draw_distance(
  state: &SurfaceState,
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
  body: bool,
) -> f32 {
  let (first, count) = (item.start_head as usize, item.end_head as usize);
  let Some(line) = first
    .checked_add(count)
    .and_then(|last| state.annotation.paths.get(first..last))
  else {
    return f32::INFINITY;
  };
  let chain: Vec<[f32; 2]> = line
    .iter()
    .map(|[x, y]| {
      let (x, y) = display_point(image, f64::from(*x), f64::from(*y));
      [x as f32, y as f32]
    })
    .collect();
  let probe = [point.0 as f32, point.1 as f32];
  let width = (item.width * image.width) as f32;
  if !body {
    return freehand_distance(probe, &chain, width);
  }
  let (low, high) = (
    display_point(image, item.start_x, item.start_y),
    display_point(image, item.end_x, item.end_y),
  );
  freehand_body_distance(
    probe,
    &chain,
    width,
    [low.0 as f32, low.1 as f32],
    [high.0 as f32, high.1 as f32],
  )
}
