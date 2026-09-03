// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn the_atlas_covers_every_character_a_readout_can_contain() {
  assert_eq!(glyph_count(), 22);
  assert_eq!(glyph_index('#'), Some(0));
  assert_eq!(glyph_index('0'), Some(1));
  assert_eq!(glyph_index('F'), Some(16));
  assert_eq!(glyph_index('×'), Some(17));
  assert_eq!(glyph_index(' '), Some(18));
  assert_eq!(glyph_index('≈'), Some(21));
  assert_eq!(glyph_index('Z'), None);
}

#[test]
fn cell_sampling_starts_inside_the_gutter_and_stops_a_texel_short() {
  // A 10px glyph in a 12px cell across 24 cells: the offset skips the
  // gutter by half a texel and the width loses one texel at the far edge.
  let (offset, width) = atlas_uv(10, 288);
  assert!((f64::from(offset) - 1.5 / 288.0).abs() < 1e-9);
  assert!((f64::from(width) - 9.0 / 288.0).abs() < 1e-9);
  // Degenerate inputs never produce a negative sampling window.
  assert_eq!(atlas_uv(0, 288).1, 0.0);
  assert_eq!(atlas_uv(10, 0), (0.0, 0.0));
}

#[test]
fn glyph_rectangles_walk_evenly_spaced_cells() {
  let metrics = AtlasMetrics {
    glyph_width: 8.0,
    u_offset: 0.01,
    u_width: 0.03,
    count: 4,
  };
  let first = metrics.glyph_texture_rect(0);
  let third = metrics.glyph_texture_rect(2);
  assert!((first.origin.x - 0.01).abs() < 1e-6);
  assert!((third.origin.x - 0.51).abs() < 1e-6);
  assert_eq!(first.size.width, third.size.width);
  assert_eq!(first.size.height, 1.0);
}

#[test]
fn coverage_is_stored_premultiplied_so_the_shader_can_divide_it_out() {
  let rgba = premultiply(&[0, 128, 255], [1.0, 0.0, 0.5]);
  assert_eq!(&rgba[0..4], &[0, 0, 0, 0]);
  assert_eq!(&rgba[4..8], &[128, 0, 64, 128]);
  assert_eq!(&rgba[8..12], &[255, 0, 128, 255]);
}
