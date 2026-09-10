// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Align an axis-aligned surface to the physical pixel grid. Rotated geometry
/// continues through `add_line` unchanged.
pub(crate) fn pixel_aligned_rect(rect: Rect, scale: f64) -> Rect {
  if scale <= 0.0 {
    return rect;
  }
  let left = (rect.origin.x * scale).round() / scale;
  let top = (rect.origin.y * scale).round() / scale;
  let right = (rect.right() * scale).round() / scale;
  let bottom = (rect.bottom() * scale).round() / scale;
  let width = (right - left).max(0.0);
  let height = (bottom - top).max(0.0);
  Rect::from_xywh(
    left,
    top,
    if rect.size.width > 0.0 {
      width.max(1.0 / scale)
    } else {
      width
    },
    if rect.size.height > 0.0 {
      height.max(1.0 / scale)
    } else {
      height
    },
  )
}

pub(crate) fn add_pixel_aligned_quad(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  scale: f64,
  kind: u32,
) {
  add_quad(out, view, pixel_aligned_rect(rect, scale), kind);
}

pub(crate) fn add_pixel_aligned_texture_quad(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  texture_rect: Rect,
  scale: f64,
  kind: u32,
) {
  add_texture_quad(
    out,
    view,
    pixel_aligned_rect(rect, scale),
    texture_rect,
    kind,
  );
}

pub(super) fn rect_corners(view: Size, rect: Rect) -> [[f32; 2]; 4] {
  [
    ndc(view, rect.origin.x, rect.origin.y),
    ndc(view, rect.right(), rect.origin.y),
    ndc(view, rect.right(), rect.bottom()),
    ndc(view, rect.origin.x, rect.bottom()),
  ]
}
