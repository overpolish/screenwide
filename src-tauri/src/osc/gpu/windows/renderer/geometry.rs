// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn ndc(view: Size, x: f64, y: f64) -> [f32; 2] {
  [
    (2.0 * x / view.width.max(1.0) - 1.0) as f32,
    (1.0 - 2.0 * y / view.height.max(1.0)) as f32,
  ]
}

pub(super) fn push_quad(
  out: &mut Vec<Vertex>,
  corners: [[f32; 2]; 4],
  uvs: [[f32; 2]; 4],
  kind: u32,
) {
  push_quad_with_aux(out, corners, uvs, [0.0; 2], kind);
}

pub(super) fn push_quad_with_aux(
  out: &mut Vec<Vertex>,
  corners: [[f32; 2]; 4],
  uvs: [[f32; 2]; 4],
  aux: [f32; 2],
  kind: u32,
) {
  let vertex = |index: usize| Vertex {
    position: corners[index],
    uv: uvs[index],
    aux,
    kind,
    padding: 0,
  };
  out.extend_from_slice(&[
    vertex(0),
    vertex(1),
    vertex(2),
    vertex(0),
    vertex(2),
    vertex(3),
  ]);
}

pub(super) fn is_empty(rect: Rect) -> bool {
  rect.size.width <= 0.0 || rect.size.height <= 0.0
}

pub(crate) fn add_quad(out: &mut Vec<Vertex>, view: Size, rect: Rect, kind: u32) {
  push_quad(out, rect_corners(view, rect), UNIT_UVS, kind);
}

pub(crate) fn add_texture_quad(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  texture_rect: Rect,
  kind: u32,
) {
  let min_u = texture_rect.origin.x as f32;
  let min_v = texture_rect.origin.y as f32;
  let max_u = texture_rect.right() as f32;
  let max_v = texture_rect.bottom() as f32;
  push_quad(
    out,
    rect_corners(view, rect),
    [
      [min_u, min_v],
      [max_u, min_v],
      [max_u, max_v],
      [min_u, max_v],
    ],
    kind,
  );
}

/// Both ends are extended by the half width so the fragment SDF can round the
/// caps without the quad clipping them.
pub(crate) fn add_line(
  out: &mut Vec<Vertex>,
  view: Size,
  start: Point,
  end: Point,
  width: f64,
  kind: u32,
) {
  let dx = end.x - start.x;
  let dy = end.y - start.y;
  let length = dx.hypot(dy);
  if length <= 0.0001 || width <= 0.0 {
    return;
  }
  let half = width * 0.5;
  let ux = dx / length;
  let uy = dy / length;
  let px = -uy * half;
  let py = ux * half;
  let extended_start = Point {
    x: start.x - ux * half,
    y: start.y - uy * half,
  };
  let extended_end = Point {
    x: end.x + ux * half,
    y: end.y + uy * half,
  };
  push_quad(
    out,
    [
      ndc(view, extended_start.x + px, extended_start.y + py),
      ndc(view, extended_end.x + px, extended_end.y + py),
      ndc(view, extended_end.x - px, extended_end.y - py),
      ndc(view, extended_start.x - px, extended_start.y - py),
    ],
    UNIT_UVS,
    kind,
  );
}
