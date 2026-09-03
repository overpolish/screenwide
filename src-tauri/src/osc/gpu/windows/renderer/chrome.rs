// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The icon atlas holds six cells; the fragment shader derives the cell from
/// `kind - 21`.
pub(crate) fn add_icon(
  out: &mut Vec<Vertex>,
  view: Size,
  icon: u8,
  left: f64,
  top: f64,
  icon_size: f64,
) {
  if icon == 0 {
    return;
  }
  add_quad(
    out,
    view,
    Rect::from_xywh(left, top, icon_size, icon_size),
    21 + u32::from(icon),
  );
}

/// A rounded chrome plate. The radius travels in `RenderConstants::chrome`
/// because it changes per control, and every control is its own draw call.
pub(crate) fn add_plate(out: &mut Vec<Vertex>, view: Size, rect: Rect) {
  if is_empty(rect) {
    return;
  }
  add_quad(out, view, rect, 46);
}

/// A chrome label quad. The texture holds white coverage; the tint is the
/// control's own foreground, pushed with the same draw call.
pub(crate) fn add_label(out: &mut Vec<Vertex>, view: Size, rect: Rect) {
  if is_empty(rect) {
    return;
  }
  add_quad(out, view, rect, 47);
}

/// A CPU-rasterised coverage label tinted by the shared OSC shader. The
/// secondary form samples t1 so paired controls share one draw contract.
pub(crate) fn add_coverage_label(out: &mut Vec<Vertex>, view: Size, rect: Rect, secondary: bool) {
  if is_empty(rect) {
    return;
  }
  add_quad(out, view, rect, if secondary { 50 } else { 49 });
}

/// A contrast-safe text readout with a theme-aware halo.
pub(crate) fn add_outlined_label(out: &mut Vec<Vertex>, view: Size, rect: Rect) {
  if is_empty(rect) {
    return;
  }
  add_quad(out, view, rect, 51);
}
