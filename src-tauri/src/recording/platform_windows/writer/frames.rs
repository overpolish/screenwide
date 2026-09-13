// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Takes ownership of window content before WGC can recycle its frame-pool
/// surface. The copy never leaves the GPU; Media Foundation may safely retain
/// samples that refer to this immutable texture while a later WGC frame is
/// being cached.
pub(in crate::recording::platform_windows) fn snapshot_frame(
  device: &ID3D11Device,
  mut frame: Frame,
) -> Result<Frame, String> {
  let mut source_description = D3D11_TEXTURE2D_DESC::default();
  unsafe { frame.texture.GetDesc(&mut source_description) };
  let description = D3D11_TEXTURE2D_DESC {
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
    CPUAccessFlags: 0,
    MiscFlags: 0,
    ..source_description
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "Direct3D created no cached window frame".to_owned())?;
  let context = unsafe { device.GetImmediateContext() }.map_err(|error| error.to_string())?;
  let source = frame
    .texture
    .cast::<ID3D11Resource>()
    .map_err(|error| error.to_string())?;
  let target = texture
    .cast::<ID3D11Resource>()
    .map_err(|error| error.to_string())?;
  unsafe { context.CopyResource(&target, &source) };
  frame.texture = texture;
  Ok(frame)
}

/// Copies a monitor-local region into an encoder-sized texture without
/// mapping either surface to the CPU.
pub(super) fn crop_frame(
  device: &ID3D11Device,
  mut frame: Frame,
  crop: CaptureRect,
) -> Result<Frame, String> {
  let mut source_description = D3D11_TEXTURE2D_DESC::default();
  unsafe { frame.texture.GetDesc(&mut source_description) };
  let description = D3D11_TEXTURE2D_DESC {
    Width: crop.width,
    Height: crop.height,
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
    CPUAccessFlags: 0,
    MiscFlags: 0,
    ..source_description
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "Direct3D created no cropped region frame".to_owned())?;
  let context = unsafe { device.GetImmediateContext() }.map_err(|error| error.to_string())?;
  let source = frame
    .texture
    .cast::<ID3D11Resource>()
    .map_err(|error| error.to_string())?;
  let target = texture
    .cast::<ID3D11Resource>()
    .map_err(|error| error.to_string())?;
  let source_box = D3D11_BOX {
    left: crop.x,
    top: crop.y,
    front: 0,
    right: crop.x.saturating_add(crop.width),
    bottom: crop.y.saturating_add(crop.height),
    back: 1,
  };
  unsafe {
    context.CopySubresourceRegion(&target, 0, 0, 0, 0, &source, 0, Some(&source_box));
  }
  frame.texture = texture;
  Ok(frame)
}
