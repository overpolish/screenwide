// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The transparent texture the selection overlay binds to every pixel-shader
//! texture slot. The overlay itself draws no textured quad, but the shared OSC
//! shader declares the slots, so each one has to hold a real view.

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

pub(super) struct PlaceholderTexture {
  /// Kept alive for the view; never read back.
  _texture: ID3D11Texture2D,
  pub(super) view: ID3D11ShaderResourceView,
}

/// Uploads a 1x1 fully transparent immutable shader-readable texture.
pub(super) fn placeholder_texture(device: &ID3D11Device) -> Result<PlaceholderTexture, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: 1,
    Height: 1,
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
  let pixels = [0u8; 4];
  let initial = D3D11_SUBRESOURCE_DATA {
    pSysMem: pixels.as_ptr().cast::<c_void>(),
    SysMemPitch: 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture =
    texture.ok_or_else(|| "D3D11 created no selection placeholder texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(PlaceholderTexture {
    _texture: texture,
    view: view.ok_or_else(|| "D3D11 created no selection placeholder view".to_owned())?,
  })
}
