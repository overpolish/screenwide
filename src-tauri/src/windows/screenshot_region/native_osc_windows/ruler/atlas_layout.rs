// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Lays atlas glyphs out from `left` and returns the covered width.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_atlas_text(
  out: &mut Vec<Vertex>,
  view: Size,
  cells: &AtlasMetrics,
  text: &str,
  left: f64,
  top: f64,
  height: f64,
  pitch: f64,
  scale: f64,
  kind: u32,
) -> f64 {
  let cell = cells.glyph_width;
  let mut x = left;
  for glyph in text.chars() {
    let index = super::super::text::glyph_index(glyph).unwrap_or(0);
    let advance = if pitch > 0.0 {
      pitch
    } else {
      cells.advance(index)
    };
    renderer::add_pixel_aligned_texture_quad(
      out,
      view,
      Rect::from_xywh(x + (advance - cell) * 0.5, top, cell, height),
      cells.glyph_texture_rect(index),
      scale,
      kind,
    );
    x += advance;
  }
  x - left
}

/// The hex readout changes with every pointer sample, so its cells share one
/// pitch. Per-glyph advances would shuffle the code's columns and shove the
/// dimensions along beside it as the sampled colour changed.
pub(super) fn hex_pitch(cells: &AtlasMetrics) -> f64 {
  cells.pitch("#0123456789ABCDEF")
}
