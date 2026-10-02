// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cursor artwork handed to the compositor rather than loaded by it: one
//! layer per style, each drawn at its own recorded size.

#[cfg(target_os = "macos")]
use std::hash::{Hash, Hasher};

use super::*;

impl Compositor {
  /// Draws the cursor from `artworks`, one per style, from now on. The set is
  /// uploaded only when it differs from the one the compositor holds.
  #[cfg(target_os = "macos")]
  pub(crate) fn use_cursor_artworks(
    &mut self,
    artworks: &[CursorArtwork<'_>],
  ) -> Result<(), String> {
    let key = fingerprint(artworks);
    if key == self.cursor_artwork_key && !self.cursor_artworks.is_empty() {
      return Ok(());
    }
    if artworks.is_empty() {
      return Err("The cursor has no artwork to draw".to_owned());
    }
    // Every layer shares the largest artwork's size; a style only ever reads
    // its own corner of its layer.
    let width = artworks
      .iter()
      .map(|artwork| artwork.size.0)
      .max()
      .unwrap_or(1)
      .max(1);
    let height = artworks
      .iter()
      .map(|artwork| artwork.size.1)
      .max()
      .unwrap_or(1)
      .max(1);
    let layer_bytes = width as usize * height as usize * 4;
    let mut layers = vec![0_u8; layer_bytes * artworks.len()];
    for (index, artwork) in artworks.iter().enumerate() {
      let row = artwork.size.0 as usize * 4;
      if artwork.size.0 == 0
        || artwork.size.1 == 0
        || artwork.pixels.len() < row * artwork.size.1 as usize
      {
        return Err("A cursor artwork does not fill its bitmap".to_owned());
      }
      let layer = &mut layers[index * layer_bytes..(index + 1) * layer_bytes];
      for (y, source) in artwork
        .pixels
        .chunks_exact(row)
        .take(artwork.size.1 as usize)
        .enumerate()
      {
        let start = y * width as usize * 4;
        layer[start..start + row].copy_from_slice(source);
      }
    }
    self.cursor_view = self
      .gpu
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
    self.cursor_artworks = artworks
      .iter()
      .map(|artwork| ArtworkStyle {
        size: artwork.size,
        design: artwork.design,
        origin: artwork.origin,
        use_design: artwork.use_design,
        clip_local_box: artwork.clip_local_box,
        supersample: artwork.supersample,
      })
      .collect();
    self.cursor_artwork_key = key;
    Ok(())
  }

  /// The artwork constants for `cursor`: the artwork model's words when the
  /// compositor holds handed artwork, and zeroes for the platform's cursors.
  pub(super) fn cursor_artwork_constants(
    &self,
    cursor: Option<crate::editor::cursor_effects::GpuCursor>,
  ) -> Result<ArtworkConstants, String> {
    let Some(cursor) = cursor.filter(|_| !self.cursor_artworks.is_empty()) else {
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

#[cfg(target_os = "macos")]
impl<'a> From<&'a crate::editor::cursor_effects::GpuArtwork> for CursorArtwork<'a> {
  fn from(artwork: &'a crate::editor::cursor_effects::GpuArtwork) -> Self {
    Self {
      pixels: &artwork.pixels,
      size: (artwork.width, artwork.height),
      design: (artwork.design_width, artwork.design_height),
      origin: (artwork.origin_x, artwork.origin_y),
      use_design: artwork.use_design,
      clip_local_box: artwork.clip_local_box,
      supersample: artwork.supersample,
    }
  }
}

/// A fingerprint of every artwork's bitmap and how it is drawn.
#[cfg(target_os = "macos")]
fn fingerprint(artworks: &[CursorArtwork<'_>]) -> u64 {
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  for artwork in artworks {
    artwork.pixels.hash(&mut hasher);
    artwork.size.hash(&mut hasher);
    artwork.design.0.to_bits().hash(&mut hasher);
    artwork.design.1.to_bits().hash(&mut hasher);
    artwork.origin.0.to_bits().hash(&mut hasher);
    artwork.origin.1.to_bits().hash(&mut hasher);
    (
      artwork.use_design,
      artwork.clip_local_box,
      artwork.supersample,
    )
      .hash(&mut hasher);
  }
  hasher.finish()
}
