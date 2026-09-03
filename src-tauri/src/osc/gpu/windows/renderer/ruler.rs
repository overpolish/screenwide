// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn add_ruler_box(
  out: &mut Vec<Vertex>,
  view: Size,
  frame: Rect,
  scale: f64,
  hovered: bool,
  hover_width: f64,
) {
  let min_x = snap(frame.origin.x, scale);
  let max_x = snap(frame.right(), scale);
  let min_y = snap(frame.origin.y, scale);
  let max_y = snap(frame.bottom(), scale);
  let halo_width = if hovered { hover_width } else { 3.0 / scale };
  let margin = halo_width * 0.5 + 1.0 / scale;
  add_quad(
    out,
    view,
    Rect::from_xywh(
      min_x - margin,
      min_y - margin,
      max_x - min_x + margin * 2.0,
      max_y - min_y + margin * 2.0,
    ),
    if hovered { 34 } else { 35 },
  );

  let half = 0.5 / scale;
  let vertical_height = (max_y - min_y - half * 2.0).max(0.0);
  add_quad(
    out,
    view,
    Rect::from_xywh(
      min_x - half,
      min_y - half,
      max_x - min_x + half * 2.0,
      half * 2.0,
    ),
    28,
  );
  add_quad(
    out,
    view,
    Rect::from_xywh(
      min_x - half,
      max_y - half,
      max_x - min_x + half * 2.0,
      half * 2.0,
    ),
    28,
  );
  if vertical_height > 0.0 {
    add_quad(
      out,
      view,
      Rect::from_xywh(min_x - half, min_y + half, half * 2.0, vertical_height),
      28,
    );
    add_quad(
      out,
      view,
      Rect::from_xywh(max_x - half, min_y + half, half * 2.0, vertical_height),
      28,
    );
  }
}

/// The uv is `(p - center) * sign / radius`, so the fragment shader always
/// sees the same unit-radius quadrant regardless of which corner this is.
fn add_ruler_arc_quad(
  out: &mut Vec<Vertex>,
  view: Size,
  center: Point,
  radius: f64,
  corner: u8,
  margin: f64,
  kind: u32,
) {
  let right = corner == 2 || corner == 4;
  let bottom = corner == 3 || corner == 4;
  let sign_x = if right { 1.0 } else { -1.0 };
  let sign_y = if bottom { 1.0 } else { -1.0 };
  let min_x = if right {
    center.x - margin
  } else {
    center.x - radius - margin
  };
  let max_x = if right {
    center.x + radius + margin
  } else {
    center.x + margin
  };
  let min_y = if bottom {
    center.y - margin
  } else {
    center.y - radius - margin
  };
  let max_y = if bottom {
    center.y + radius + margin
  } else {
    center.y + margin
  };
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
    kind,
  );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_ruler_arc(
  out: &mut Vec<Vertex>,
  view: Size,
  center: Point,
  radius: f64,
  corner: u8,
  scale: f64,
  hovered: bool,
  hover_width: f64,
  low_confidence: bool,
) {
  if radius <= 0.0 || scale <= 0.0 {
    return;
  }
  let center = Point {
    x: snap(center.x, scale),
    y: snap(center.y, scale),
  };
  let radius = ((radius * scale).round() / scale).max(1.0 / scale);
  if hovered {
    let margin = hover_width * 0.5 + 1.0 / scale;
    add_ruler_arc_quad(out, view, center, radius, corner, margin, 40);
  }
  add_ruler_arc_quad(
    out,
    view,
    center,
    radius,
    corner,
    1.5 / scale,
    if low_confidence { 41 } else { 39 },
  );
}
