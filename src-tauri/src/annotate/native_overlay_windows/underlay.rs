// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a highlight is recoloured from, as a texture the overlay's shader
//! reads at `t10`: a display's underlay, or the still a highlight is baked
//! into.

use super::*;

use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11Resource, D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
  D3D11_USAGE_IMMUTABLE,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};

use crate::screenshots::CapturedImage;

/// A display's underlay as the surface holds it: the view the shader reads,
/// and which of Rust's revisions it is.
pub(super) struct HeldUnderlay {
  pub(super) view: ID3D11ShaderResourceView,
  pub(super) revision: u64,
}

/// `image` uploaded as a straight RGBA texture the shader can read.
pub(super) fn upload(
  device: &ID3D11Device,
  image: &CapturedImage,
) -> Result<ID3D11ShaderResourceView, String> {
  if image.width == 0 || image.height == 0 {
    return Err("A highlight's underlay has no pixels".to_owned());
  }
  let mut texture = None;
  unsafe {
    device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Width: image.width,
        Height: image.height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
          Count: 1,
          Quality: 0,
        },
        Usage: D3D11_USAGE_IMMUTABLE,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
      },
      Some(&D3D11_SUBRESOURCE_DATA {
        pSysMem: image.rgba.as_ptr().cast::<c_void>(),
        SysMemPitch: image.width * 4,
        SysMemSlicePitch: 0,
      }),
      Some(&mut texture),
    )
  }
  .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no highlight underlay".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  view.ok_or_else(|| "D3D11 created no highlight underlay view".to_owned())
}

impl Surface {
  /// The underlay this display's highlights read, uploaded again when Rust
  /// has made a new one; `None` where no highlight has been drawn on it.
  pub(super) fn underlay(
    &mut self,
    device: &ID3D11Device,
  ) -> Result<Option<ID3D11ShaderResourceView>, String> {
    let Some(current) = super::super::highlight::underlay(self.display as usize) else {
      return Ok(None);
    };
    if self.underlay.as_ref().map(|held| held.revision) != Some(current.revision) {
      self.underlay = Some(HeldUnderlay {
        view: upload(device, &current.image)?,
        revision: current.revision,
      });
    }
    Ok(self.underlay.as_ref().map(|held| held.view.clone()))
  }
}
