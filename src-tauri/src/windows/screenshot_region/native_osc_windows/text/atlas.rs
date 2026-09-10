// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  atlas_uv, glyph_count, glyph_index, premultiply, upload, AtlasMetrics, Context, TextTexture,
  ATLAS_CELLS, GUTTER, HEX_GLYPHS, LABEL_ALPHA, LIGHT_INK, SUPERSAMPLE,
};
use crate::osc::geometry::Size;
use windows::Win32::Graphics::Direct3D11::ID3D11Device;

pub(super) fn build_atlas(
  device: &ID3D11Device,
  scale: f64,
  font_size: f64,
  line_height: f64,
  ink: [f32; 3],
) -> Option<TextTexture> {
  if scale <= 0.0 {
    return None;
  }
  let raster_scale = scale * f64::from(SUPERSAMPLE);
  // Inter's figures are tabular by default, so the digits already share one
  // advance; the handful of symbols around them are narrower or wider. The
  // texture cell takes the widest advance and centres each glyph in it, so
  // one uv rectangle shape serves every cell, while callers step on screen by
  // the glyph's own advance.
  let context = Context::new(font_size, raster_scale, false)?;
  let (_, extent_y) = context.measure(HEX_GLYPHS)?;
  let count = glyph_count();
  let advances = HEX_GLYPHS
    .chars()
    .map(|glyph| {
      context
        .measure(&glyph.to_string())
        .map_or(0.0, |(extent, _)| f64::from(extent) / raster_scale)
    })
    .collect::<Vec<_>>();
  let mut cells = [0.0_f64; ATLAS_CELLS];
  for (cell, advance) in cells.iter_mut().zip(advances.iter()) {
    *cell = *advance;
  }
  let glyph_width = advances.iter().copied().fold(0.0_f64, f64::max).ceil();
  let glyph_pixel_width = (glyph_width * scale).ceil().max(1.0) as i32;
  let cell_pixel_width = glyph_pixel_width + GUTTER * 2;
  let point_height = line_height.ceil().max(1.0);
  let pixel_height = (point_height * scale).round().max(1.0) as i32;
  let pixel_width = cell_pixel_width * count as i32;
  let top = (pixel_height * SUPERSAMPLE - extent_y) / 2;
  let runs = HEX_GLYPHS
    .chars()
    .enumerate()
    .map(|(index, glyph)| {
      let centring = ((glyph_width - advances[index]) * 0.5 * raster_scale).round() as i32;
      (
        glyph.to_string(),
        (index as i32 * cell_pixel_width + GUTTER) * SUPERSAMPLE + centring,
        top,
      )
    })
    .collect::<Vec<_>>();
  let mut coverage = context.coverage((pixel_width, pixel_height), &runs)?;
  // NumberField's unit decoration: content-fg-secondary, light / dark.
  let unit_alpha = if ink == LIGHT_INK { 0.50 } else { 0.55 };
  for (offset, coverage) in coverage.iter_mut().enumerate() {
    let cell = (offset % pixel_width as usize) / cell_pixel_width as usize;
    if cell == glyph_index('p')? || cell == glyph_index('x')? {
      *coverage = (f32::from(*coverage) * unit_alpha / LABEL_ALPHA).round() as u8;
    }
  }
  drop(context);
  let view = upload(
    device,
    &premultiply(&coverage, ink),
    pixel_width,
    pixel_height,
  )?;
  let (u_offset, u_width) = atlas_uv(glyph_pixel_width, pixel_width);
  Some(TextTexture {
    view,
    size: Size {
      width: advances.iter().sum(),
      height: point_height,
    },
    atlas: Some(AtlasMetrics {
      glyph_width,
      advances: cells,
      u_offset,
      u_width,
      count,
    }),
  })
}
