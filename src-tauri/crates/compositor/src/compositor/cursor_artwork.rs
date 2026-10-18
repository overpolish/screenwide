// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cursor's artwork: every style's bitmap in one texture array, a layer
//! each, each drawn at its own recorded size.

use super::*;
use screenwide_cursor_effects::GpuArtwork;

/// The array `artworks` are uploaded into, and what the shader needs of each.
pub(super) fn upload(
  gpu: &Gpu,
  artworks: &[GpuArtwork],
) -> Result<(wgpu::TextureView, Vec<ArtworkStyle>), String> {
  if artworks.is_empty() {
    return Err("The cursor has no artwork to draw".to_owned());
  }
  // Every layer shares the largest artwork's size; a style only ever reads
  // its own corner of its layer.
  let width = artworks
    .iter()
    .map(|artwork| artwork.width)
    .max()
    .unwrap_or(1)
    .max(1);
  let height = artworks
    .iter()
    .map(|artwork| artwork.height)
    .max()
    .unwrap_or(1)
    .max(1);
  let layer_bytes = width as usize * height as usize * 4;
  let mut layers = vec![0_u8; layer_bytes * artworks.len()];
  for (index, artwork) in artworks.iter().enumerate() {
    let row = artwork.width as usize * 4;
    if artwork.width == 0
      || artwork.height == 0
      || artwork.pixels.len() < row * artwork.height as usize
    {
      return Err("A cursor artwork does not fill its bitmap".to_owned());
    }
    let layer = &mut layers[index * layer_bytes..(index + 1) * layer_bytes];
    for (y, source) in artwork
      .pixels
      .chunks_exact(row)
      .take(artwork.height as usize)
      .enumerate()
    {
      let start = y * width as usize * 4;
      layer[start..start + row].copy_from_slice(source);
    }
  }
  let view = gpu
    .texture_with_pixels(
      "Screenwide cursor artwork",
      (width, height, artworks.len() as u32),
      wgpu::TextureFormat::Rgba8Unorm,
      &layers,
    )
    .create_view(&wgpu::TextureViewDescriptor {
      dimension: Some(wgpu::TextureViewDimension::D2Array),
      ..Default::default()
    });
  let styles = artworks
    .iter()
    .map(|artwork| ArtworkStyle {
      size: (artwork.width, artwork.height),
      design: (artwork.design_width, artwork.design_height),
      origin: (artwork.origin_x, artwork.origin_y),
      use_design: artwork.use_design,
      clip_local_box: artwork.clip_local_box,
      supersample: artwork.supersample,
    })
    .collect();
  Ok((view, styles))
}

impl Compositor {
  /// The artwork constants for `cursor`, zeroes where there is none.
  pub(super) fn cursor_artwork_constants(
    &self,
    cursor: Option<screenwide_cursor_effects::GpuCursor>,
  ) -> Result<ArtworkConstants, String> {
    let Some(cursor) = cursor else {
      return Ok(([0.0; 4], [0.0; 4], [0; 4]));
    };
    let style = self
      .cursor_artworks
      .get(cursor.style as usize)
      .ok_or_else(|| "The cursor's style has no artwork".to_owned())?;
    Ok((
      [
        style.size.0 as f32,
        style.size.1 as f32,
        style.design.0,
        style.design.1,
      ],
      [
        cursor.hotspot_x,
        cursor.hotspot_y,
        style.origin.0,
        style.origin.1,
      ],
      [
        1,
        u32::from(style.use_design),
        u32::from(style.clip_local_box),
        u32::from(style.supersample),
      ],
    ))
  }
}
