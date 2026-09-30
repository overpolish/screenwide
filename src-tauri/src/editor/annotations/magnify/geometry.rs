// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A magnifier's prepared record, and the distances that pick it.
//!
//! The record carries, in the pixels it was prepared in: the loupe's centre
//! in `a`, its half size in `b` and its corner radius in `rounding`, as this
//! frame draws it; the zoom area's centre in `c`, its half size in
//! `start_head.c` and its corner radius in `high`; the rim's pen in `width`;
//! how present the magnifier is in `low`, which its shadow deepens by; the
//! line from the zoom area's edge to the loupe's in `start_head.a` to
//! `start_head.b`, with `head` one where there is room for it; and the
//! loupe's grip, at the middle of its right side, in `end_head.c`.
//!
//! As a clip arrives, the loupe comes out of the zoom area: it starts as the
//! zoom area itself, showing what it covers at its own size, and grows as it
//! travels to its place, so the enlargement arrives with it and the loupe
//! lifts off the picture rather than fading in over it. `reveal.scale` is
//! how far it has travelled.

use crate::editor::annotations::geometry::{ArrowGeometry, ArrowTriangle};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::rounded_box_distance;

/// A box rounded at its corners, by its centre and half size.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RoundedBox {
  centre: [f32; 2],
  half: [f32; 2],
  radius: f32,
}

impl RoundedBox {
  fn distance(self, point: [f32; 2]) -> f32 {
    rounded_box_distance(
      point,
      [self.centre[0] - self.half[0], self.centre[1] - self.half[1]],
      [self.centre[0] + self.half[0], self.centre[1] + self.half[1]],
      self.radius,
    )
  }

  /// Where the way from the centre to `towards` leaves the box, or `None`
  /// where `towards` lies inside it.
  fn exit(self, towards: [f32; 2]) -> Option<[f32; 2]> {
    let along = |t: f32| lerp(self.centre, towards, t);
    if self.distance(towards) <= 0.0 {
      return None;
    }
    let (mut inside, mut outside) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
      let middle = (inside + outside) * 0.5;
      if self.distance(along(middle)) <= 0.0 {
        inside = middle;
      } else {
        outside = middle;
      }
    }
    Some(along(outside))
  }
}

/// The corner radius `percent` of a box's shorter side gives it.
fn rounding(half: [f32; 2], percent: f32) -> f32 {
  2.0 * half[0].min(half[1]).max(0.0) * share(percent)
}

/// A radius percentage as a share of the shorter side, from 0 to a half.
fn share(percent: f32) -> f32 {
  if percent.is_finite() {
    percent.clamp(0.0, 50.0) / 100.0
  } else {
    0.0
  }
}

fn lerp(from: [f32; 2], to: [f32; 2], t: f32) -> [f32; 2] {
  [
    from[0] + (to[0] - from[0]) * t,
    from[1] + (to[1] - from[1]) * t,
  ]
}

/// Where a loupe's grip sits per unit of its longer side, measured from its
/// centre: the middle of its right side. The loupe is the zoom area's `half`
/// scaled up, and no rounding reaches the middle of a side, so the grip is on
/// the rim at every size; `magnify::gesture` reads a dragged grip back
/// through it.
pub(crate) fn unit_grip(half: [f32; 2]) -> [f32; 2] {
  let longest = (half[0].max(half[1]) * 2.0).max(f32::EPSILON);
  [half[0] / longest, 0.0]
}

/// The magnifier with its zoom area from `start` to `end` and its loupe
/// centred on `loupe`, `size` along its longer side, in the space the three
/// are given in, rounded by `radius` percent of the shorter side, its rim
/// drawn `width` wide, at `reveal`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_magnify(
  start: [f32; 2],
  loupe: [f32; 2],
  end: [f32; 2],
  size: f32,
  radius: f32,
  width: f32,
  reveal: AnnotationReveal,
) -> ArrowGeometry {
  let low = [start[0].min(end[0]), start[1].min(end[1])];
  let high = [start[0].max(end[0]), start[1].max(end[1])];
  let area_half = [(high[0] - low[0]) * 0.5, (high[1] - low[1]) * 0.5];
  let area = RoundedBox {
    centre: [(low[0] + high[0]) * 0.5, (low[1] + high[1]) * 0.5],
    half: area_half,
    radius: rounding(area_half, radius),
  };
  let longest = area_half[0].max(area_half[1]) * 2.0;
  let zoom = if longest > 0.0 && size.is_finite() {
    size.max(0.0) / longest
  } else {
    0.0
  };
  let whole_half = [area_half[0] * zoom, area_half[1] * zoom];
  let travel = if reveal.scale.is_finite() {
    reveal.scale.clamp(0.0, 1.0)
  } else {
    1.0
  };
  let half = lerp(area.half, whole_half, travel);
  let drawn = RoundedBox {
    centre: lerp(area.centre, loupe, travel),
    half,
    radius: rounding(half, radius),
  };
  // Where one box lies inside the other there is no gap for the line to
  // cross, and it would only be drawn under the loupe.
  let line = area
    .exit(drawn.centre)
    .zip(drawn.exit(area.centre))
    .filter(|(from, _)| drawn.distance(*from) > 0.0);
  let [from, to] = line.map_or([[0.0; 2]; 2], |(from, to)| [from, to]);
  let whole_grip = unit_grip(area_half);
  ArrowGeometry {
    a: drawn.centre,
    b: drawn.half,
    c: area.centre,
    width: width.max(0.0),
    low: reveal.opacity.clamp(0.0, 1.0),
    high: area.radius,
    start_head: ArrowTriangle {
      a: from,
      b: to,
      c: area.half,
    },
    end_head: ArrowTriangle {
      a: [0.0, 0.0],
      b: [0.0, 0.0],
      c: [
        loupe[0] + whole_grip[0] * size,
        loupe[1] + whole_grip[1] * size,
      ],
    },
    rounding: drawn.radius,
    head: u32::from(line.is_some()),
  }
}

/// How far `point` falls outside the loupe or the zoom area, whichever is
/// nearer: zero or less anywhere inside either, so a press on the loupe or
/// on what it enlarges picks the magnifier.
pub(crate) fn magnify_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  let (loupe, area) = boxes(geometry);
  loupe.distance(point).min(area.distance(point))
}

/// Which part of a magnifier `point` is on: the loupe, which is drawn over
/// the zoom area where the two meet, or the zoom area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MagnifyPart {
  Loupe,
  Area,
}

pub(crate) fn magnify_part(point: [f32; 2], geometry: &ArrowGeometry) -> Option<MagnifyPart> {
  let (loupe, area) = boxes(geometry);
  if loupe.distance(point) <= 0.0 {
    Some(MagnifyPart::Loupe)
  } else if area.distance(point) <= 0.0 {
    Some(MagnifyPart::Area)
  } else {
    None
  }
}

fn boxes(geometry: &ArrowGeometry) -> (RoundedBox, RoundedBox) {
  (
    RoundedBox {
      centre: geometry.a,
      half: geometry.b,
      radius: geometry.rounding,
    },
    RoundedBox {
      centre: geometry.c,
      half: geometry.start_head.c,
      radius: geometry.high,
    },
  )
}
