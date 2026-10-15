// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the overlay draws of a turned layer: its frame carried through the
//! turn, the gizmo's rings, and the turn its annotation chrome is placed
//! through.

use super::*;
use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_osc::gpu::pipeline::TURN_RING_ROWS;
use screenwide_osc::turn::ring_basis;

/// The rings' vertex kind in `osc.wgsl`, one quad drawing all three.
pub(in super::super) const RING_KIND: u32 = 60;
/// A ring's coloured core, half across, and its dark outline, in points.
const RING_CORE: f64 = 2.0;
const RING_OUTLINE: f64 = 1.0;
/// How far the ring under the pointer, and the ring being dragged, are lit
/// toward white.
const RING_HOVER_LIGHT: f32 = 0.15;
const RING_DRAG_LIGHT: f32 = 0.35;
/// How strongly the rings' far halves show.
const RING_FAR_ALPHA: f32 = 0.3;

/// The rings' constants rows and their quad, in device pixels.
pub(in super::super) struct Rings {
  pub(in super::super) rows: [[f32; 4]; TURN_RING_ROWS],
  pub(in super::super) quad: [f32; 4],
}

/// What the overlay draws of a turned layer: its frame, corners clockwise
/// from its own top-left then its radius dot, in device pixels; the rings;
/// and the turn, in display points, the annotation chrome is carried through.
#[derive(Default)]
pub(in super::super) struct TurnChrome {
  pub(in super::super) frame: Option<[[f32; 2]; 5]>,
  pub(in super::super) rings: Option<Rings>,
  pub(in super::super) annotations: Option<Homography>,
}

/// What the overlay draws of the selected layer's turn.
pub(in super::super) fn chrome(state: &SurfaceState, scale: f64) -> TurnChrome {
  let turn = state
    .selection
    .and_then(|selection| homographies(state, selection).map(|(forward, _)| (selection, forward)));
  TurnChrome {
    frame: turn.and_then(|(selection, forward)| turned_frame(state, selection, forward, scale)),
    rings: rings(state, scale),
    annotations: turn.map(|(_, forward)| forward),
  }
}

fn turned_frame(
  state: &SurfaceState,
  selection: PreviewSelection,
  forward: Homography,
  scale: f64,
) -> Option<[[f32; 2]; 5]> {
  if selection.crop_mode != 0 {
    return None;
  }
  let frame = display_selection(state, selection)?;
  let (right, bottom) = (frame.x + frame.width, frame.y + frame.height);
  // The radius dot sits where the upright selection draws it.
  let points = [
    (frame.x, frame.y),
    (right, frame.y),
    (right, bottom),
    (frame.x, bottom),
    radius_point(frame, selection.radius_percent),
  ];
  Some(points.map(|point| {
    let (x, y) = carry(forward, point);
    [(x * scale) as f32, (y * scale) as f32]
  }))
}

/// The rings shown: the chosen image's, or else the selected layer's.
fn rings(state: &SurfaceState, scale: f64) -> Option<Rings> {
  let (turn, (x, y), radius) = match annotation::image_gizmo(state) {
    Some(gizmo) => (gizmo.seen, gizmo.centre, gizmo.radius),
    None if rings_shown(state) => {
      let (centre, radius) = ring_frame(state)?;
      (rotation(state.selection?), centre, radius)
    }
    None => return None,
  };
  let dragged = state
    .turn
    .drag
    .map(|held| held.drag.axis as usize)
    .or(state.turn.image.map(|held| held.axis()));
  let held = dragged.or(state.turn.hovered);
  let mut rows = [[0.0_f32; 4]; TURN_RING_ROWS];
  // Each direction scaled to the ring in the target's pixels; the depth only
  // gives the side, so it stays as it is.
  for (row, [along_x, along_y, depth]) in rows.iter_mut().zip(ring_basis(turn)) {
    *row = [
      (along_x * radius * scale) as f32,
      (along_y * radius * scale) as f32,
      depth as f32,
      0.0,
    ];
  }
  rows[6] = [
    (x * scale) as f32,
    (y * scale) as f32,
    (RING_CORE * scale) as f32,
    (RING_OUTLINE * scale) as f32,
  ];
  rows[7] = [
    held.map_or(-1.0, |axis| axis as f32),
    if dragged.is_some() {
      RING_DRAG_LIGHT
    } else {
      RING_HOVER_LIGHT
    },
    RING_FAR_ALPHA,
    0.0,
  ];
  // The rings and a pixel round them for the shader to anti-alias across.
  let reach = radius + RING_CORE + RING_OUTLINE + 1.0 / scale;
  Some(Rings {
    rows,
    quad: [
      ((x - reach) * scale) as f32,
      ((y - reach) * scale) as f32,
      (reach * 2.0 * scale) as f32,
      (reach * 2.0 * scale) as f32,
    ],
  })
}

/// `point`, on the selected layer standing upright, carried through
/// `forward`.
pub(in super::super) fn carried_point(forward: Homography, point: Point) -> Point {
  let (x, y) = carry(forward, (point.x, point.y));
  Point { x, y }
}

/// An upright rect's corners, clockwise from its top-left, carried through
/// `forward`.
pub(in super::super) fn carried_rect(forward: Homography, rect: Rect) -> [Point; 4] {
  let (left, top) = (rect.origin.x, rect.origin.y);
  let (right, bottom) = (rect.right(), rect.bottom());
  [(left, top), (right, top), (right, bottom), (left, bottom)]
    .map(|(x, y)| carried_point(forward, Point { x, y }))
}

/// An upright box's corners, clockwise from its top-left, and its radius dot
/// where `radius_percent` gives one, as the layer selection places it,
/// carried through `forward`. A turned layer's box annotation wears these as
/// soft lines: its upright chrome is pixel-snapped quads, which carried
/// through the turn would step along every side.
pub(in super::super) fn carried_box(
  forward: Homography,
  frame: Rect,
  radius_percent: Option<f64>,
) -> ([Point; 4], Option<Point>) {
  let dot = radius_percent.map(|percent| {
    let offset = frame.size.width.min(frame.size.height) * percent / 100.0 * 0.55 + 10.0;
    carried_point(
      forward,
      Point {
        x: frame.origin.x + offset,
        y: frame.origin.y + offset,
      },
    )
  });
  (carried_rect(forward, frame), dot)
}

/// How far past the view a carried guide may run, in points, so its caps
/// stay off screen.
const GUIDE_OVERRUN: f64 = 8.0;

/// The segment from `from` to `to`, on the selected layer standing upright,
/// carried through `forward` and cut to `view`: `None` where none of it is
/// both in front of the eye and on screen. A guide runs the whole view, and
/// the turn can tip part of that behind the eye or fling it far off screen,
/// where its ends would lose all precision on the way to the GPU.
pub(in super::super) fn carried_segment(
  forward: Homography,
  from: Point,
  to: Point,
  view: Size,
) -> Option<(Point, Point)> {
  // The carried weight runs linearly along the upright segment, so the part
  // in front of the eye is one stretch of it.
  let weight = |point: Point| forward[2][0] * point.x + forward[2][1] * point.y + forward[2][2];
  let (w0, w1) = (weight(from), weight(to));
  let least = 1e-6 * w0.max(w1);
  if w0.max(w1) <= 0.0 {
    return None;
  }
  let at = |t: f64| Point {
    x: from.x + (to.x - from.x) * t,
    y: from.y + (to.y - from.y) * t,
  };
  let cut = |w: f64, other: f64| (least - w) / (other - w);
  let (start, end) = (
    if w0 < least { cut(w0, w1) } else { 0.0 },
    if w1 < least { 1.0 - cut(w1, w0) } else { 1.0 },
  );
  let (a, b) = (
    carried_point(forward, at(start)),
    carried_point(forward, at(end)),
  );
  clip_to(a, b, view)
}

/// The part of the segment from `a` to `b` inside `view`, grown by
/// [`GUIDE_OVERRUN`] all round.
fn clip_to(a: Point, b: Point, view: Size) -> Option<(Point, Point)> {
  let (dx, dy) = (b.x - a.x, b.y - a.y);
  let (mut low, mut high) = (0.0_f64, 1.0_f64);
  for (p, q) in [
    (-dx, a.x + GUIDE_OVERRUN),
    (dx, view.width + GUIDE_OVERRUN - a.x),
    (-dy, a.y + GUIDE_OVERRUN),
    (dy, view.height + GUIDE_OVERRUN - a.y),
  ] {
    if p == 0.0 {
      if q < 0.0 {
        return None;
      }
      continue;
    }
    let t = q / p;
    if p < 0.0 {
      low = low.max(t);
    } else {
      high = high.min(t);
    }
  }
  let at = |t: f64| Point {
    x: a.x + dx * t,
    y: a.y + dy * t,
  };
  (low < high).then_some((at(low), at(high)))
}
