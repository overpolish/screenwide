// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Draw-ready arrow geometry, prepared once per annotation before drawing or
//! picking, never per pixel.
//!
//! Both backends prepare here: the D3D11 one calls this directly and the
//! Metal compositor and the macOS chrome reach it through
//! `screenwide_annotation_prepare`, so the two draw the same pixels from the
//! same annotation. The shaders hold the only other copy of the shape, as a
//! per-pixel SDF over the record this fills.

use crate::editor::annotations::geometry::{add, bezier, distance, length, scale, subtract};
use crate::editor::annotations::geometry::{ArrowGeometry, ArrowTriangle};
use crate::editor::annotations::reveal::geometry::reveal_geometry;
use crate::editor::annotations::reveal::AnnotationReveal;

/// Where the shaft has to stop for a head of `length` to meet it: the
/// parameter in `[low, high]` at which the curve leaves `tip` by that much.
/// A static arrow searches its own half of the curve, so a head can never eat
/// the whole shaft; a revealing one searches the window it is showing, where
/// a head legitimately fills everything there is.
#[allow(clippy::too_many_arguments)]
fn trim(
  a: [f32; 2],
  b: [f32; 2],
  c: [f32; 2],
  tip: [f32; 2],
  length: f32,
  at_end: bool,
  mut low: f32,
  mut high: f32,
) -> f32 {
  let limit = length * length;
  for _ in 0..10 {
    let t = (low + high) * 0.5;
    let offset = subtract(bezier(a, b, c, t), tip);
    if ((offset[0] * offset[0] + offset[1] * offset[1]) <= limit) == at_end {
      high = t;
    } else {
      low = t;
    }
  }
  if at_end {
    high
  } else {
    low
  }
}

/// Shrink the rounded head to its inner triangle. Distance to these vertices
/// minus rounding is the final outline; both picking and the shader use it.
/// The apex extension compensates for rounding so the visible tip stays put.
fn prepare_head(tip: [f32; 2], join: [f32; 2], half_base: f32, rounding: f32) -> ArrowTriangle {
  let delta = subtract(tip, join);
  let length = length(delta);
  let direction = if length > 1e-6 {
    scale(delta, 1.0 / length)
  } else {
    [1.0, 0.0]
  };
  let k = (rounding / half_base.max(1e-4)).clamp(0.0, 0.9);
  let span = length - rounding;
  let complement = 1.0 - k * k;
  let height =
    (span + k * (span * span + half_base * half_base * complement).sqrt()) / complement.max(1e-6);
  let apex = add(tip, scale(direction, (height - length).max(0.0)));
  let across = [-direction[1] * half_base, direction[0] * half_base];
  let left = add(join, across);
  let right = subtract(join, across);
  let side_apex = distance(left, right);
  let side_left = distance(apex, right);
  let side_right = distance(apex, left);
  let perimeter = (side_apex + side_left + side_right).max(1e-6);
  let incentre = scale(
    add(
      add(scale(apex, side_apex), scale(left, side_left)),
      scale(right, side_right),
    ),
    1.0 / perimeter,
  );
  let area = 0.5
    * ((left[0] - apex[0]) * (right[1] - apex[1]) - (right[0] - apex[0]) * (left[1] - apex[1]))
      .abs();
  let inradius = 2.0 * area / perimeter;
  let shrink = (inradius - rounding).max(0.0) / inradius.max(1e-6);
  ArrowTriangle {
    a: add(incentre, scale(subtract(apex, incentre), shrink)),
    b: add(incentre, scale(subtract(left, incentre), shrink)),
    c: add(incentre, scale(subtract(right, incentre), shrink)),
  }
}

/// One preparation per annotation before drawing or picking. A short curve
/// scales the stroke and heads together. `reveal` is how much of the annotation
/// this frame draws: the whole path at full size prepares exactly what a still
/// always has; an annotation part way through its clip wears each head on its
/// end of the shaft, riding the end that is moving.
pub(crate) fn prepare_arrow(
  a: [f32; 2],
  b: [f32; 2],
  c: [f32; 2],
  width: f32,
  head: u32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let mut result = ArrowGeometry {
    a,
    b,
    c,
    width: width.max(0.0),
    low: 0.0,
    high: 1.0,
    head,
    ..ArrowGeometry::default()
  };
  let middle = bezier(a, b, c, 0.5);
  if head != 0 {
    let room = distance(middle, c).max(distance(bezier(a, b, c, 0.75), c));
    result.width = result.width.min(room * 0.8 / 4.0);
  }
  if head == 2 {
    let room = distance(middle, a).max(distance(bezier(a, b, c, 0.25), a));
    result.width = result.width.min(room * 0.8 / 4.0);
  }
  // Coincident handles draw a round dot, avoiding a degenerate triangle SDF.
  if result.width <= 1e-4 {
    result.width = width.max(0.0);
    result.head = 0;
    return result;
  }
  let travel = reveal_geometry(a, b, c, result.width, head as f32, reveal);
  let revealing = travel.low > 0.0 || travel.high < 1.0 || travel.scale < 1.0;
  let scale = travel.scale;
  let length = result.width * 4.0 * scale;
  let half_base = result.width * 2.0 * scale;
  result.rounding = result.width * 0.35 * scale;
  // Stroke, heads and rounding are one annotation and scale together.
  result.width *= scale;
  result.low = travel.low;
  result.high = travel.high;
  // A head the reveal has not grown to half a pixel yet has no triangle worth
  // building: its three vertices collapse onto each other, and a triangle
  // with no winding reads as inside everywhere.
  if revealing && length <= 0.5 {
    result.head = 0;
    return result;
  }
  // A growing head narrows fast, so while an annotation is revealing, the shaft
  // runs on a little way under the head rather than stopping at its base.
  let pullback = 0.88;
  if head != 0 {
    let tip = if revealing {
      bezier(a, b, c, travel.end_tip)
    } else {
      c
    };
    let join = if revealing {
      travel.high
    } else {
      trim(a, b, c, tip, length, true, 0.5, 1.0)
    };
    result.end_head = prepare_head(tip, bezier(a, b, c, join), half_base, result.rounding);
    result.high = if revealing {
      trim(
        a,
        b,
        c,
        tip,
        length * pullback,
        true,
        travel.high,
        travel.end_tip,
      )
    } else {
      join
    };
  }
  if head == 2 {
    let tip = if revealing {
      bezier(a, b, c, travel.start_tip)
    } else {
      a
    };
    let join = if revealing {
      travel.low
    } else {
      trim(a, b, c, tip, length, false, 0.0, 0.5)
    };
    result.start_head = prepare_head(tip, bezier(a, b, c, join), half_base, result.rounding);
    result.low = if revealing {
      trim(
        a,
        b,
        c,
        tip,
        length * pullback,
        false,
        travel.start_tip,
        travel.low,
      )
    } else {
      join
    };
  }
  result
}

#[cfg(test)]
mod tests {
  use super::super::distance::head_distance;
  use super::*;
  #[test]
  fn a_straight_arrow_keeps_its_tip_and_trims_its_shaft() {
    let arrow = prepare_arrow(
      [0.0, 0.0],
      [100.0, 0.0],
      [200.0, 0.0],
      8.0,
      1,
      AnnotationReveal::WHOLE,
    );
    assert_eq!(arrow.head, 1);
    assert_eq!(arrow.width, 8.0);
    assert_eq!(arrow.low, 0.0);
    // The shaft stops one head length short of the tip.
    let join = bezier(arrow.a, arrow.b, arrow.c, arrow.high);
    assert!((join[0] - 168.0).abs() < 0.5, "{join:?}");
    // The visible tip stays on the end point: the inner apex sits behind it
    // by exactly the rounding.
    assert!((arrow.end_head.a[0] + arrow.rounding - 200.0).abs() < 0.05);
    assert_eq!(
      head_distance([199.0, 0.0], arrow.end_head, arrow.rounding),
      0.0
    );
    assert!(head_distance([230.0, 0.0], arrow.end_head, arrow.rounding) > 20.0);
  }

  #[test]
  fn a_short_arrow_scales_down_and_coincident_handles_draw_a_dot() {
    let short = prepare_arrow(
      [0.0, 0.0],
      [5.0, 0.0],
      [10.0, 0.0],
      8.0,
      1,
      AnnotationReveal::WHOLE,
    );
    assert!(short.width < 8.0);
    let dot = prepare_arrow(
      [3.0, 3.0],
      [3.0, 3.0],
      [3.0, 3.0],
      8.0,
      2,
      AnnotationReveal::WHOLE,
    );
    assert_eq!(dot.head, 0);
    assert_eq!(dot.width, 8.0);
  }
}
