// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Chrome on a turned picture, whose sides need not run along the pixel
//! grid: soft lines, each quad padded a pixel either side for the shader to
//! anti-alias across, rather than the snapped quads upright chrome is drawn
//! with. The selection's own frame with its grips, a frame alone, a marquee
//! band and a guide's line. The twins of the `screenwide_region_osc_add_turned_*`
//! functions and `screenwide_region_osc_add_soft_guide`.

use super::*;

/// The pixel either side of a soft line's quad that its edge fades across.
fn pad(scale: f64) -> f64 {
  2.0 / scale
}

/// The selection's outline along a turned box's four sides, without its
/// grips: the dark halo, then the light core. `corners` run clockwise from
/// the box's own top-left.
pub fn add_turned_outline(out: &mut Vec<Vertex>, view: Size, corners: [Point; 4], scale: f64) {
  for (width, kind) in [(3.0 / scale, 51), (1.0 / scale, 50)] {
    for side in 0..4 {
      add_line(
        out,
        view,
        corners[side],
        corners[(side + 1) % 4],
        width + pad(scale),
        kind,
      );
    }
  }
}

/// A marquee band's marching ants along a turned band's four sides, each
/// side's pattern starting at its own first corner as an upright band's
/// starts at its corner.
pub fn add_turned_marquee(out: &mut Vec<Vertex>, view: Size, corners: [Point; 4], scale: f64) {
  let width = 3.0 / scale + pad(scale);
  let half = width * 0.5;
  for side in 0..4 {
    let (from, to) = (corners[side], corners[(side + 1) % 4]);
    let span = (to.x - from.x).hypot(to.y - from.y);
    // The quad runs half its width past each corner, so the periods are
    // counted from there to keep the pattern anchored on the corner.
    let periods = |distance: f64| (distance * scale / 12.0) as f32;
    add_line_along(
      out,
      view,
      from,
      to,
      width,
      63,
      [periods(-half), periods(span + half)],
    );
  }
}

/// A snap guide's line from `from` to `to`, `width` across: the canvas's own
/// line, or another object's where `object` is set. A turned layer's guides,
/// gap bars and snapped element's outline are these.
pub fn add_soft_guide(
  out: &mut Vec<Vertex>,
  view: Size,
  from: Point,
  to: Point,
  width: f64,
  object: bool,
  scale: f64,
) {
  add_line(
    out,
    view,
    from,
    to,
    width + pad(scale),
    if object { 62 } else { 61 },
  );
}

/// A turned picture's own frame: the selection's outline along its four
/// sides, its corner discs and the pills on its sides lying along them.
/// `corners` run clockwise from the picture's own top-left.
pub fn add_turned_selection(out: &mut Vec<Vertex>, view: Size, corners: [Point; 4], scale: f64) {
  add_turned_outline(out, view, corners, scale);
  let radius = 4.0 + 1.0 / scale;
  // A side's pill lies along it, the length and thickness of an upright one.
  let length = 12.0 + 4.0 / scale;
  let thickness = 6.0 + 4.0 / scale;
  let middles = side_middles(corners);
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
    let middle = middles[side];
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
}

/// Where the middle of each side lands, side `n` running from `corners[n]`
/// to the next. Seen in perspective that is not halfway between the side's
/// ends: the far half of a side is drawn shorter than the near half. A press
/// on a pill is read back through the turn onto the upright picture, where
/// it is tested against the side's true middle, so the pill is drawn there.
/// The one projection that takes a square onto `corners` is the turn itself,
/// whatever the picture's own proportions, so the corners alone place it.
fn side_middles(corners: [Point; 4]) -> [Point; 4] {
  let halfway = |side: usize| {
    let (from, to) = (corners[side], corners[(side + 1) % 4]);
    Point {
      x: (from.x + to.x) * 0.5,
      y: (from.y + to.y) * 0.5,
    }
  };
  let [p0, p1, p2, p3] = corners;
  let (dx1, dy1) = (p1.x - p2.x, p1.y - p2.y);
  let (dx2, dy2) = (p3.x - p2.x, p3.y - p2.y);
  let (dx3, dy3) = (p0.x - p1.x + p2.x - p3.x, p0.y - p1.y + p2.y - p3.y);
  let det = dx1 * dy2 - dx2 * dy1;
  if det.abs() < 1e-9 {
    return std::array::from_fn(halfway);
  }
  // The square's own (u, v) to the screen, after Heckbert's square-to-quad
  // mapping; flat on screen, `g` and `h` are zero and this is affine.
  let g = (dx3 * dy2 - dx2 * dy3) / det;
  let h = (dx1 * dy3 - dx3 * dy1) / det;
  let (a, b) = (p1.x - p0.x + g * p1.x, p3.x - p0.x + h * p3.x);
  let (d, e) = (p1.y - p0.y + g * p1.y, p3.y - p0.y + h * p3.y);
  let at = |side: usize, (u, v): (f64, f64)| {
    let w = g * u + h * v + 1.0;
    if w <= 1e-9 {
      return halfway(side);
    }
    Point {
      x: (a * u + b * v + p0.x) / w,
      y: (d * u + e * v + p0.y) / w,
    }
  };
  [
    at(0, (0.5, 0.0)),
    at(1, (1.0, 0.5)),
    at(2, (0.5, 1.0)),
    at(3, (0.0, 0.5)),
  ]
}

#[cfg(test)]
mod tests;
