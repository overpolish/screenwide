// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The objects picked in the camera, drawn in the selection overlay while
//! they are picked, as `recording_preview_surface_macos+object_picking_osc.m`
//! draws them on macOS: a dot for each click and the line for each painting,
//! each haloed in its own colour where it is chosen.

use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_osc::gpu::pipeline::PICKED_POINT_PAIRS;
use screenwide_osc::gpu::windows::{add_line, add_texture_quad, RenderConstants, Vertex};

/// How far the halo round the chosen object reaches past it, in points: as
/// far as an annotation's hover halo grows.
const HALO: f64 = 8.0;
/// A dot's radius, the size of a moment's dot on the timeline.
const DOT_RADIUS: f64 = 4.0;

/// The objects picked in the frame shown, in the overlay's logical points.
pub(in crate::editor::preview_platform::surface_windows) struct ObjectChrome {
  pub(super) objects: Vec<Object>,
}

/// One object: the click, or the points of the line painted over it.
pub(super) struct Object {
  pub(super) points: Vec<Point>,
  pub(super) removed: bool,
  pub(super) chosen: bool,
}

/// Adds `chrome` to `out`, handing the halo round a chosen painted line to
/// the shader in `constants`.
pub(in crate::editor::preview_platform::surface_windows) fn add_objects(
  out: &mut Vec<Vertex>,
  constants: &mut RenderConstants,
  view: Size,
  chrome: &ObjectChrome,
  scale: f64,
) {
  let whole = Rect::from_xywh(0.0, 0.0, 1.0, 1.0);
  let pad = 2.0 / scale;
  for object in &chrome.objects {
    let [first, ..] = object.points.as_slice() else {
      continue;
    };
    if object.points.len() > 1 {
      if object.chosen {
        add_line_halo(out, constants, view, &object.points, object.removed, scale);
      }
      // A line three points wide in the object's colour over the controls'
      // outline, which keeps it apart from the camera behind it.
      for (width, kind) in [(5.0, 51), (3.0, if object.removed { 56 } else { 55 })] {
        for pair in object.points.windows(2) {
          add_line(out, view, pair[0], pair[1], width + pad, kind);
        }
      }
      continue;
    }
    // Its centre on a whole device pixel, over its halo where it is chosen.
    let x = (first.x * scale).round() / scale;
    let y = (first.y * scale).round() / scale;
    let disc = |radius: f64| Rect::from_xywh(x - radius, y - radius, radius * 2.0, radius * 2.0);
    if object.chosen {
      let kind = if object.removed { 58 } else { 54 };
      add_texture_quad(out, view, disc(DOT_RADIUS + HALO), whole, kind);
    }
    let kind = if object.removed { 53 } else { 52 };
    add_texture_quad(out, view, disc(DOT_RADIUS), whole, kind);
  }
}

/// The halo round the chosen line, in its colour: one quad over the line and
/// the reach round it, inside which the shader measures each pixel to the
/// line through `points`, handed over in `constants` in device pixels.
fn add_line_halo(
  out: &mut Vec<Vertex>,
  constants: &mut RenderConstants,
  view: Size,
  points: &[Point],
  removed: bool,
  scale: f64,
) {
  let reach = 1.5 + HALO;
  let room = PICKED_POINT_PAIRS * 2;
  let shown = points.len().min(room);
  let (mut left, mut top, mut right, mut bottom) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
  for index in 0..shown {
    // Spread over the line where it has more points than there is room for.
    let point = points[if shown > 1 {
      index * (points.len() - 1) / (shown - 1)
    } else {
      0
    }];
    let slot = &mut constants.picked_points[index / 2][(index % 2) * 2..][..2];
    slot.copy_from_slice(&[(point.x * scale) as f32, (point.y * scale) as f32]);
    left = left.min(point.x - reach);
    top = top.min(point.y - reach);
    right = right.max(point.x + reach);
    bottom = bottom.max(point.y + reach);
  }
  constants.picked_halo = [(reach * scale) as f32, shown as f32, 0.0, 0.0];
  let bounds = Rect::from_xywh(
    left - 1.0,
    top - 1.0,
    right - left + 2.0,
    bottom - top + 2.0,
  );
  let kind = if removed { 59 } else { 57 };
  add_texture_quad(out, view, bounds, Rect::from_xywh(0.0, 0.0, 1.0, 1.0), kind);
}
