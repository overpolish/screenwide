// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Shades the sliver between a crop corner and the rounded corner the layer
/// will actually have.
///
/// The four shade quads stop at the crop rectangle, so with a corner radius
/// the little wedge inside the rectangle but outside the rounded shape would
/// read as kept. `uv` is the distance from the arc centre in radii, which lets
/// the fragment shade exactly what falls outside the arc.
fn add_corner_shade(
  out: &mut Vec<Vertex>,
  view: Size,
  center: Point,
  radius: f64,
  sign_x: f64,
  sign_y: f64,
) {
  if radius <= 0.0 {
    return;
  }
  let outer_x = center.x + radius * sign_x;
  let outer_y = center.y + radius * sign_y;
  let min_x = center.x.min(outer_x);
  let max_x = center.x.max(outer_x);
  let min_y = center.y.min(outer_y);
  let max_y = center.y.max(outer_y);
  let uv = |x: f64, y: f64| {
    [
      ((x - center.x) * sign_x / radius) as f32,
      ((y - center.y) * sign_y / radius) as f32,
    ]
  };
  push_quad(
    out,
    [
      ndc(view, min_x, min_y),
      ndc(view, max_x, min_y),
      ndc(view, max_x, max_y),
      ndc(view, min_x, max_y),
    ],
    [
      uv(min_x, min_y),
      uv(max_x, min_y),
      uv(max_x, max_y),
      uv(min_x, max_y),
    ],
    45,
  );
}

/// `radius_percent` is the layer's corner radius as a percentage of the crop
/// rectangle's shorter side, so the shade rounds with the result.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_crop_with_handles(
  out: &mut Vec<Vertex>,
  view: Size,
  crop: Rect,
  image: Rect,
  scale: f64,
  radius_percent: f64,
  show_frame: bool,
  show_handles: bool,
) {
  let shade = [
    Rect::from_xywh(
      image.origin.x,
      image.origin.y,
      image.size.width,
      (crop.origin.y - image.origin.y).max(0.0),
    ),
    Rect::from_xywh(
      image.origin.x,
      crop.bottom(),
      image.size.width,
      (image.bottom() - crop.bottom()).max(0.0),
    ),
    Rect::from_xywh(
      image.origin.x,
      crop.origin.y,
      (crop.origin.x - image.origin.x).max(0.0),
      crop.size.height,
    ),
    Rect::from_xywh(
      crop.right(),
      crop.origin.y,
      (image.right() - crop.right()).max(0.0),
      crop.size.height,
    ),
  ];
  for rect in shade {
    if !is_empty(rect) {
      add_quad(out, view, rect, 6);
    }
  }

  let shortest = crop.size.width.min(crop.size.height);
  let corner_radius = radius_percent.clamp(0.0, 50.0) / 100.0 * shortest;
  if corner_radius > 0.0 {
    let left = crop.origin.x + corner_radius;
    let right = crop.right() - corner_radius;
    let top = crop.origin.y + corner_radius;
    let bottom = crop.bottom() - corner_radius;
    for (x, y, sign_x, sign_y) in [
      (left, top, -1.0, -1.0),
      (right, top, 1.0, -1.0),
      (left, bottom, -1.0, 1.0),
      (right, bottom, 1.0, 1.0),
    ] {
      add_corner_shade(out, view, Point { x, y }, corner_radius, sign_x, sign_y);
    }
  }

  if !show_frame {
    return;
  }

  let min_x = snap(crop.origin.x, scale);
  let max_x = snap(crop.right(), scale);
  let min_y = snap(crop.origin.y, scale);
  let max_y = snap(crop.bottom(), scale);
  let mid_x = snap((min_x + max_x) / 2.0, scale);
  let mid_y = snap((min_y + max_y) / 2.0, scale);
  let half = 1.5 / scale;
  let width_pixels = (max_x - min_x) * scale;
  let height_pixels = (max_y - min_y) * scale;
  let perimeter = (width_pixels + height_pixels) * 2.0;
  let cycles = (perimeter / 12.0).round().max(1.0);
  let pattern_scale = cycles * 12.0 / perimeter.max(1.0);
  let pattern_width = width_pixels * pattern_scale;
  let pattern_height = height_pixels * pattern_scale;
  add_pattern_quad(
    out,
    view,
    Rect::from_xywh(min_x, min_y - half, max_x - min_x, half * 2.0),
    PatternEdge {
      kind: 8,
      horizontal: true,
      forward: true,
      phase: 0.0,
      length: pattern_width,
    },
  );
  add_pattern_quad(
    out,
    view,
    Rect::from_xywh(min_x, max_y - half, max_x - min_x, half * 2.0),
    PatternEdge {
      kind: 8,
      horizontal: true,
      forward: false,
      phase: pattern_width + pattern_height,
      length: pattern_width,
    },
  );
  add_pattern_quad(
    out,
    view,
    Rect::from_xywh(min_x - half, min_y, half * 2.0, max_y - min_y),
    PatternEdge {
      kind: 10,
      horizontal: false,
      forward: false,
      phase: pattern_width * 2.0 + pattern_height,
      length: pattern_height,
    },
  );
  add_pattern_quad(
    out,
    view,
    Rect::from_xywh(max_x - half, min_y, half * 2.0, max_y - min_y),
    PatternEdge {
      kind: 10,
      horizontal: false,
      forward: true,
      phase: pattern_width,
      length: pattern_height,
    },
  );
  if show_handles {
    add_handles(
      out,
      view,
      handle_points(min_x, min_y, max_x, max_y, mid_x, mid_y),
      scale,
    );
  }
}

pub(crate) fn add_crop(
  out: &mut Vec<Vertex>,
  view: Size,
  crop: Rect,
  image: Rect,
  scale: f64,
  radius_percent: f64,
) {
  add_crop_with_handles(out, view, crop, image, scale, radius_percent, true, true);
}
