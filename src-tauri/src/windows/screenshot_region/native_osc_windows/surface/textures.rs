// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn upload_icons(device: &ID3D11Device) -> Result<ID3D11ShaderResourceView, String> {
  let atlas = unsafe { screenwide_osc_icon_atlas() };
  let expected = atlas.width as usize * atlas.height as usize;
  if atlas.pixels.is_null() || atlas.length < expected || expected == 0 {
    return Err("the shared OSC icon atlas is empty".to_owned());
  }
  let pixels = unsafe { std::slice::from_raw_parts(atlas.pixels, expected) };
  upload_texture(
    device,
    pixels,
    atlas.width,
    atlas.height,
    DXGI_FORMAT_R8_UNORM,
    1,
  )
}

pub(in crate::windows::screenshot_region::native_osc_windows) fn upload_rgba(
  device: &ID3D11Device,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> Result<ID3D11ShaderResourceView, String> {
  upload_texture(device, rgba, width, height, DXGI_FORMAT_R8G8B8A8_UNORM, 4)
}

pub(super) fn upload_texture(
  device: &ID3D11Device,
  rgba: &[u8],
  width: u32,
  height: u32,
  format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
  bytes_per_pixel: u32,
) -> Result<ID3D11ShaderResourceView, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let data = D3D11_SUBRESOURCE_DATA {
    pSysMem: rgba.as_ptr().cast::<c_void>(),
    SysMemPitch: width * bytes_per_pixel,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&data), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no region OSC texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  view.ok_or_else(|| "D3D11 created no region OSC texture view".to_owned())
}
