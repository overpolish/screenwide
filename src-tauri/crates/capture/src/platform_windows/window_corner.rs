// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How round a recorded window's corners are, read off its first frame:
//! Windows draws a window's rounded corners clear in its capture, as macOS
//! does in a still of a window alone, and the movie keeps them black. The
//! editor cuts the picture to that radius (`corner_radius`).

use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11Device, ID3D11Resource, ID3D11Texture2D, D3D11_BOX, D3D11_CPU_ACCESS_READ,
  D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};

use crate::desktop_capture::corner_radius;

/// How far into the corner the window is looked at, in pixels.
const SPAN: u32 = 96;

/// The radius, in pixels, of the top start corner of the window in `frame`,
/// which fills `filled` of it; none for a square corner or one that cannot
/// be read.
pub(super) fn measure(
  device: &ID3D11Device,
  frame: &ID3D11Texture2D,
  filled: (u32, u32),
) -> Option<f32> {
  let span = SPAN.min(filled.0 / 2).min(filled.1 / 2);
  if span == 0 {
    return None;
  }
  let mut source_description = D3D11_TEXTURE2D_DESC::default();
  unsafe { frame.GetDesc(&mut source_description) };
  let description = D3D11_TEXTURE2D_DESC {
    Width: span,
    Height: span,
    MipLevels: 1,
    ArraySize: 1,
    Usage: D3D11_USAGE_STAGING,
    BindFlags: 0,
    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
    MiscFlags: 0,
    ..source_description
  };
  let mut staging = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut staging)) }.ok()?;
  let staging = staging?;
  let context = unsafe { device.GetImmediateContext() }.ok()?;
  let source: ID3D11Resource = frame.cast().ok()?;
  let target: ID3D11Resource = staging.cast().ok()?;
  let corner = D3D11_BOX {
    left: 0,
    top: 0,
    front: 0,
    right: span,
    bottom: span,
    back: 1,
  };
  unsafe { context.CopySubresourceRegion(&target, 0, 0, 0, 0, &source, 0, Some(&corner)) };
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  // Mapping waits for the copy above.
  unsafe { context.Map(&target, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) }.ok()?;
  let stride = mapped.RowPitch as usize;
  // SAFETY: the mapping covers `span` rows of `RowPitch` bytes, BGRA, and
  // is read only until it is unmapped below.
  let pixels =
    unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), stride * span as usize) };
  let radius = corner_radius(
    |x, y| pixels[y as usize * stride + x as usize * 4 + 3] < 128,
    span,
  );
  unsafe { context.Unmap(&target, 0) };
  radius.map(|radius| radius as f32)
}
