// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immutable D3D11 textures used by selection-overlay labels.

use std::ffi::c_void;

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
      D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
      D3D11_USAGE_IMMUTABLE,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

/// The rasterised "W × H" readout and the inputs it was built from. Rebuilding
/// the bitmap on every gesture sample would burn a GDI text layout per pointer
/// event, so the texture is reused until the text or the scale changes. The
/// theme is applied in the shader, so it needs no rebuild.
pub(super) struct LabelTexture {
  /// Kept alive for the view; never read back.
  _texture: ID3D11Texture2D,
  pub(super) action: bool,
  pub(super) scale_key: u32,
  pub(super) size: (u32, u32),
  pub(super) text: String,
  pub(super) view: ID3D11ShaderResourceView,
}

/// Uploads `pixels` (BGRA, `size`) as an immutable shader-readable texture.
pub(super) fn upload_label_texture(
  device: &ID3D11Device,
  pixels: &[u8],
  size: (u32, u32),
  text: &str,
  scale_key: u32,
  action: bool,
) -> Result<LabelTexture, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.0,
    Height: size.1,
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
    pSysMem: pixels.as_ptr().cast::<c_void>(),
    SysMemPitch: size.0 * 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no selection label texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(LabelTexture {
    _texture: texture,
    action,
    scale_key,
    size,
    text: text.to_owned(),
    view: view.ok_or_else(|| "D3D11 created no selection label view".to_owned())?,
  })
}
