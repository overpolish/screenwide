// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The textures the redaction passes draw into and read back.

use super::*;
use windows::Win32::Graphics::Direct3D11::D3D11_BIND_RENDER_TARGET;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

/// A texture a pass draws into and the next one reads.
pub(super) struct Target {
  pub(super) texture: ID3D11Texture2D,
  pub(super) view: ID3D11ShaderResourceView,
  pub(super) target: ID3D11RenderTargetView,
}

pub(super) fn target(
  device: &ID3D11Device,
  size: (u32, u32),
  format: DXGI_FORMAT,
) -> Result<Target, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.0,
    Height: size.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0) as u32,
    ..Default::default()
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| format!("The redaction target could not be created: {error}"))?;
  let texture = texture.ok_or_else(|| "D3D11 created no redaction target".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let (mut view, mut target) = (None, None);
  unsafe {
    device
      .CreateShaderResourceView(&resource, None, Some(&mut view))
      .and_then(|()| device.CreateRenderTargetView(&resource, None, Some(&mut target)))
  }
  .map_err(|error| error.to_string())?;
  Ok(Target {
    texture,
    view: view.ok_or_else(|| "D3D11 created no redaction view".to_owned())?,
    target: target.ok_or_else(|| "D3D11 created no redaction target view".to_owned())?,
  })
}

/// A target a pass draws into before the paint pass reads it, grown to fit
/// the largest it has been asked for, and never shrunk.
#[derive(Default)]
pub(super) struct Scratch(std::sync::Mutex<Option<((u32, u32), Target)>>);

impl Scratch {
  /// `draw` into the target, first grown to at least `size` in `format`, and
  /// its view.
  pub(super) fn draw(
    &self,
    device: &ID3D11Device,
    size: (u32, u32),
    format: DXGI_FORMAT,
    draw: impl FnOnce(&Target),
  ) -> Result<ID3D11ShaderResourceView, String> {
    let mut slot = self
      .0
      .lock()
      .map_err(|_| "A redaction scratch target is poisoned".to_owned())?;
    if slot
      .as_ref()
      .is_none_or(|(held, _)| held.0 < size.0 || held.1 < size.1)
    {
      let grown = slot
        .as_ref()
        .map_or(size, |(held, _)| (held.0.max(size.0), held.1.max(size.1)));
      *slot = Some((grown, target(device, grown, format)?));
    }
    let (_, target) = slot.as_ref().expect("the scratch target was just made");
    draw(target);
    Ok(target.view.clone())
  }
}
