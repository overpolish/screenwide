// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decoded frames in, for the shared compositor: Media Foundation decodes on
//! Direct3D 11, and each frame is copied into a texture both devices open.

use windows::Win32::Graphics::Direct3D11::{ID3D11Resource, ID3D11Texture2D};

use super::{Compositor, SourceTexture, FORMAT};
use screenwide_gpu::D3d11Layer;

impl Compositor {
  /// A source decoded frames are copied into: a BGRA texture Direct3D 11,
  /// which the decoder writes on, can copy into too.
  pub fn source(&self, d3d11: &D3d11Layer, size: (u32, u32)) -> Result<SourceTexture, String> {
    let shared = d3d11.shared_texture(self.gpu(), size, FORMAT, "preview source")?;
    Ok(SourceTexture {
      size,
      texture: shared.texture.clone(),
      view: shared.view.clone(),
      shared: Some(std::sync::Arc::new(shared)),
      picture: None,
      enhanced: false,
      cut_out: false,
    })
  }

  /// Copies a decoded frame into `destination`, submitted ahead of the next
  /// wgpu submission on the shared queue.
  pub fn copy_source(
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
    let source: ID3D11Resource =
      windows::core::Interface::cast(source).map_err(|error| error.to_string())?;
    d3d11.with(&[shared], |context, textures| {
      let destination: ID3D11Resource =
        windows::core::Interface::cast(&textures[0]).map_err(|error| error.to_string())?;
      unsafe {
        context.CopySubresourceRegion(&destination, 0, 0, 0, 0, &source, subresource, None);
      }
      Ok(())
    })?
  }
}
