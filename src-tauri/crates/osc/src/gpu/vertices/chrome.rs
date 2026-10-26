// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A rounded chrome plate. The radius travels in `RenderConstants::chrome`
/// because it changes per plate, and every plate is its own draw call.
pub fn add_plate(out: &mut Vec<Vertex>, view: Size, rect: Rect) {
  if is_empty(rect) {
    return;
  }
  add_quad(out, view, rect, 46);
}
