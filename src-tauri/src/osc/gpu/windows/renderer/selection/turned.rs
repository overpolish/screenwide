// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A turned picture's own frame: the selection's outline along its four
//! sides, its corner discs, the pills on its sides lying along them, and its
//! radius dot. The twin of `screenwide_region_osc_add_turned_selection`.

use super::*;

/// `corners` run clockwise from the picture's own top-left. Its sides need
/// not run along the pixel grid, so they are soft lines, each quad padded a
/// pixel either side for the shader to anti-alias across, rather than
/// snapped quads.
pub(crate) fn add_turned_selection(
  out: &mut Vec<Vertex>,
  view: Size,
  corners: [Point; 4],
  radius_dot: Point,
  scale: f64,
) {
  let pad = 2.0 / scale;
  for (width, kind) in [(3.0 / scale, 51), (1.0 / scale, 50)] {
    for side in 0..4 {
      add_line(
        out,
        view,
        corners[side],
        corners[(side + 1) % 4],
        width + pad,
        kind,
      );
    }
  }
  let radius = 4.0 + 1.0 / scale;
  // A side's pill lies along it, the length and thickness of an upright one.
  let length = 12.0 + 4.0 / scale;
  let thickness = 6.0 + 4.0 / scale;
  for side in 0..4 {
    let (from, to) = (corners[side], corners[(side + 1) % 4]);
    add_circle(
      out,
      view,
      snap_handle_point(from, scale),
      radius,
      1.0 / scale,
      3,
    );
    let span = (to.x - from.x).hypot(to.y - from.y);
    if span <= 0.0001 {
      continue;
    }
    let reach = (length - thickness) * 0.5 / span;
    let middle = Point {
      x: (from.x + to.x) * 0.5,
      y: (from.y + to.y) * 0.5,
    };
    let along = Point {
      x: (to.x - from.x) * reach,
      y: (to.y - from.y) * reach,
    };
    add_line(
      out,
      view,
      Point {
        x: middle.x - along.x,
        y: middle.y - along.y,
      },
      Point {
        x: middle.x + along.x,
        y: middle.y + along.y,
      },
      thickness,
      16,
    );
  }
  add_circle(
    out,
    view,
    snap_handle_point(radius_dot, scale),
    radius,
    1.0 / scale,
    3,
  );
}
