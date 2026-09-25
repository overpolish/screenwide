// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  pub(in crate::editor::preview_platform::surface) fn source(
    &self,
    device: &ID3D11Device,
    size: (u32, u32),
  ) -> Result<SourceTexture, String> {
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
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
      ..Default::default()
    };
    let mut texture = None;
    unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
      .map_err(|error| error.to_string())?;
    let texture = texture.ok_or_else(|| "D3D11 created no preview source texture".to_owned())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut view = None;
    unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
      .map_err(|error| error.to_string())?;
    Ok(SourceTexture {
      size,
      texture,
      view: view.ok_or_else(|| "D3D11 created no preview source view".to_owned())?,
      picture: None,
    })
  }

  pub(in crate::editor::preview_platform::surface) fn screenshot_source(
    &self,
    device: &ID3D11Device,
    source: &crate::screenshots::CapturedImage,
  ) -> Result<SourceTexture, String> {
    if source.rgba.len() != source.width as usize * source.height as usize * 4 {
      return Err("The screenshot preview source pixels are not valid".to_owned());
    }
    let description = D3D11_TEXTURE2D_DESC {
      Width: source.width,
      Height: source.height,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_R8G8B8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
      ..Default::default()
    };
    let data = D3D11_SUBRESOURCE_DATA {
      pSysMem: source.rgba.as_ptr().cast::<c_void>(),
      SysMemPitch: source.width * 4,
      SysMemSlicePitch: 0,
    };
    let mut texture = None;
    unsafe { device.CreateTexture2D(&description, Some(&data), Some(&mut texture)) }
      .map_err(|error| error.to_string())?;
    let texture =
      texture.ok_or_else(|| "D3D11 created no screenshot preview texture".to_owned())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut view = None;
    unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
      .map_err(|error| error.to_string())?;
    let view = view.ok_or_else(|| "D3D11 created no screenshot preview view".to_owned())?;
    Ok(SourceTexture {
      size: (source.width, source.height),
      texture,
      view,
      picture: None,
    })
  }

  pub(in crate::editor::preview_platform::surface) fn copy_source(
    context: &ID3D11DeviceContext,
    destination: &SourceTexture,
    source: &ID3D11Texture2D,
    subresource: u32,
  ) -> Result<(), String> {
    let destination: ID3D11Resource = destination
      .texture
      .cast()
      .map_err(|error| error.to_string())?;
    let source: ID3D11Resource = source.cast().map_err(|error| error.to_string())?;
    unsafe {
      context.CopySubresourceRegion(&destination, 0, 0, 0, 0, &source, subresource, None);
    }
    Ok(())
  }
}
