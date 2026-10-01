// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  /// A source decoded frames are copied into: a BGRA texture Direct3D 11,
  /// which the decoder writes on, can copy into too.
  pub(in crate::editor::preview_platform::surface) fn source(
    &self,
    d3d11: &D3d11Layer,
    size: (u32, u32),
  ) -> Result<SourceTexture, String> {
    let shared = d3d11.shared_texture(self.gpu, size, FORMAT, "preview source")?;
    Ok(SourceTexture {
      size,
      texture: shared.texture.clone(),
      view: shared.view.clone(),
      shared: Some(std::sync::Arc::new(shared)),
      picture: None,
    })
  }

  pub(in crate::editor::preview_platform::surface) fn screenshot_source(
    &self,
    source: &crate::screenshots::CapturedImage,
  ) -> Result<SourceTexture, String> {
    if source.width == 0
      || source.height == 0
      || source.rgba.len() != source.width as usize * source.height as usize * 4
    {
      return Err("The screenshot preview source pixels are not valid".to_owned());
    }
    let texture = self.gpu.texture_with_pixels(
      "Screenwide screenshot source",
      (source.width, source.height, 1),
      wgpu::TextureFormat::Rgba8Unorm,
      &source.rgba,
    );
    Ok(SourceTexture {
      size: (source.width, source.height),
      view: texture.create_view(&Default::default()),
      texture,
      shared: None,
      picture: None,
    })
  }

  /// Copies a decoded frame into `destination`, submitted ahead of the next
  /// wgpu submission on the shared queue.
  pub(in crate::editor::preview_platform::surface) fn copy_source(
    &self,
    d3d11: &D3d11Layer,
    destination: &SourceTexture,
    source: &ID3D11Texture2D,
    subresource: u32,
  ) -> Result<(), String> {
    let shared = destination
      .shared
      .as_ref()
      .ok_or_else(|| "A screenshot source cannot take a decoded frame".to_owned())?;
    let source: windows::Win32::Graphics::Direct3D11::ID3D11Resource =
      windows::core::Interface::cast(source).map_err(|error| error.to_string())?;
    d3d11.with(&[shared], |context, textures| {
      let destination: windows::Win32::Graphics::Direct3D11::ID3D11Resource =
        windows::core::Interface::cast(&textures[0]).map_err(|error| error.to_string())?;
      unsafe {
        context.CopySubresourceRegion(&destination, 0, 0, 0, 0, &source, subresource, None);
      }
      Ok(())
    })?
  }
}
