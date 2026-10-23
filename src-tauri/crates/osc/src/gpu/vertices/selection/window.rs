// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The window being picked: one quad over the whole screen (kind 64) that
/// shades, and blurs, everything but the window, with no frame, as there is
/// nothing to edit. The window's shape is in `RenderConstants::window_cutout`
/// and `window_shape`.
pub fn add_window_highlight(out: &mut Vec<Vertex>, view: Size, screen: Rect) {
  add_quad(out, view, screen, 64);
}
