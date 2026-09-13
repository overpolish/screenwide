// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl KeyboardArtworkCache {
  pub(in crate::editor::preview_platform::surface) fn visible_bounds(
    &self,
    device: &ID3D11Device,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    Ok(
      self
        .resolve(device, overlay, output.1)?
        .and_then(|(_, values)| visible_bounds::calculate(&values, output)),
    )
  }

  pub(in crate::editor::preview_platform::surface) fn resolve(
    &self,
    device: &ID3D11Device,
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
        let artwork = std::sync::Arc::new(upload(device, &raster)?);
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

fn upload(device: &ID3D11Device, raster: &KeyboardRaster) -> Result<KeyboardArtwork, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: raster.size.0,
    Height: raster.size.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_IMMUTABLE,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let initial = D3D11_SUBRESOURCE_DATA {
    pSysMem: raster.pixels.as_ptr().cast::<c_void>(),
    SysMemPitch: raster.size.0 * 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no keyboard artwork texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(KeyboardArtwork {
    _texture: texture,
    bytes: raster.pixels.len(),
    keys: raster.keys.clone(),
    size: raster.size,
    view: view.ok_or_else(|| "D3D11 created no keyboard artwork view".to_owned())?,
  })
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
