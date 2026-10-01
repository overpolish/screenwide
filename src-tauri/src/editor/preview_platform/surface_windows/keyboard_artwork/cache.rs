// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl KeyboardArtworkCache {
  pub(in crate::editor::preview_platform::surface_windows) fn visible_bounds(
    &self,
    gpu: &crate::gpu::Gpu,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    Ok(
      self
        .resolve(gpu, overlay, output.1)?
        .and_then(|(_, values)| visible_bounds::calculate(&values, output)),
    )
  }

  pub(in crate::editor::preview_platform::surface_windows) fn resolve(
    &self,
    gpu: &crate::gpu::Gpu,
    overlay: &KeyboardOverlay,
    output_height: u32,
  ) -> Result<Option<(std::sync::Arc<KeyboardArtwork>, KeyboardConstants)>, String> {
    if overlay.key_count == 0 {
      return Ok(None);
    }
    let prepared = prepared_shortcut(overlay);
    if prepared.is_empty() {
      return Ok(None);
    }
    let backing_scale = keyboard_backing_scale(output_height, overlay);
    let mut cache_key = format!("{}|{backing_scale:.0}|", overlay.appearance);
    for (code, _) in &prepared {
      cache_key.push_str(&format!("{code}:"));
    }
    let mut entries = self
      .entries
      .lock()
      .map_err(|_| "The keyboard artwork cache is unavailable".to_owned())?;
    let artwork = match entries.get(&cache_key) {
      Some(artwork) => std::sync::Arc::clone(artwork),
      None => {
        let labels = prepared
          .iter()
          .map(|(code, _)| key_label(*code))
          .collect::<Vec<_>>();
        let raster = rasterize_keyboard(
          &labels,
          overlay.appearance == KeyboardOverlay::APPEARANCE_LIGHT,
          backing_scale,
        )?;
        let artwork = std::sync::Arc::new(upload(gpu, &raster));
        let cached_bytes = entries.values().map(|entry| entry.bytes).sum::<usize>();
        if entries.len() >= CACHE_ENTRIES || cached_bytes + artwork.bytes > CACHE_BYTES {
          entries.clear();
        }
        entries.insert(cache_key, std::sync::Arc::clone(&artwork));
        artwork
      }
    };
    let mut values = KeyboardConstants {
      dimensions: [artwork.size.0, artwork.size.1, 0, 0],
      ..Default::default()
    };
    for (index, (x, width)) in artwork.keys.iter().enumerate().take(prepared.len()) {
      values.key_geometry[index][0] = *x;
      values.key_geometry[index][1] = *width;
    }
    update_uniforms(&mut values, overlay, &prepared);
    Ok(Some((artwork, values)))
  }
}

fn upload(gpu: &crate::gpu::Gpu, raster: &KeyboardRaster) -> KeyboardArtwork {
  let texture = gpu.texture_with_pixels(
    "Screenwide keyboard artwork",
    (raster.size.0, raster.size.1, 1),
    wgpu::TextureFormat::Bgra8Unorm,
    &raster.pixels,
  );
  KeyboardArtwork {
    bytes: raster.pixels.len(),
    keys: raster.keys.clone(),
    size: raster.size,
    view: texture.create_view(&Default::default()),
  }
}

/// Copies the animation state of every prepared key into the shader uniforms.
fn update_uniforms(
  values: &mut KeyboardConstants,
  overlay: &KeyboardOverlay,
  prepared: &[(u16, KeyboardKey)],
) {
  values.dimensions[2] = prepared.len() as u32;
  values.dimensions[3] = overlay.animation;
  values.animation = [
    overlay.scale,
    overlay.progress,
    overlay.maximum_width,
    overlay.requested_scale,
  ];
  values.position = [overlay.center_x, overlay.center_y, 0.0, 0.0];
  for (index, (_, state)) in prepared.iter().enumerate() {
    values.key_geometry[index][2] = state.visible;
    values.key_geometry[index][3] = state.slot;
    values.key_motion[index] = [
      state.alpha,
      state.scale,
      state.progress,
      state.layout_progress,
    ];
    values.key_masks[index] = [state.layout_from_mask, state.layout_to_mask, 0, 0];
    values.key_position[index] = [state.center_x, state.center_y, state.scale_ratio, 0.0];
  }
}
