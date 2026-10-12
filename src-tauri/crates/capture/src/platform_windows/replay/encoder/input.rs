// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Frames for the software encoder, which reads system memory.

use super::*;

pub(super) fn staging_texture(
  device: &ID3D11Device,
  size: Size,
) -> Result<ID3D11Texture2D, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.width,
    Height: size.height,
    MipLevels: 1,
    ArraySize: 1,
    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_NV12,
    SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_STAGING,
    BindFlags: 0,
    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
    MiscFlags: 0,
  };
  let mut texture = None;
  win(unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) })?;
  texture.ok_or_else(|| "Direct3D created no staging texture".to_owned())
}

/// The NV12 picture in `texture`, copied into system memory with no row
/// padding, as the software encoder reads it.
pub(super) fn read_back(
  context: &ID3D11DeviceContext,
  staging: &ID3D11Texture2D,
  texture: &ID3D11Texture2D,
  size: Size,
) -> Result<IMFMediaBuffer, String> {
  let source = win(texture.cast::<ID3D11Resource>())?;
  let target = win(staging.cast::<ID3D11Resource>())?;
  unsafe { context.CopyResource(&target, &source) };
  let (width, height) = (size.width as usize, size.height as usize);
  let length = width * height * 3 / 2;
  let buffer = win(unsafe { MFCreateMemoryBuffer(length as u32) })?;
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  win(unsafe { context.Map(&target, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) })?;
  let mut destination = std::ptr::null_mut();
  let locked = unsafe { buffer.Lock(&mut destination, None, None) };
  if let Err(error) = locked {
    unsafe { context.Unmap(&target, 0) };
    return Err(error.to_string());
  }
  let pitch = mapped.RowPitch as usize;
  // SAFETY: the mapped NV12 surface holds `height * 3 / 2` rows of `pitch`
  // bytes, the luma plane then the interleaved chroma plane, and the locked
  // buffer holds `length` bytes.
  unsafe {
    let source = mapped.pData.cast::<u8>();
    for row in 0..height * 3 / 2 {
      std::ptr::copy_nonoverlapping(source.add(row * pitch), destination.add(row * width), width);
    }
    let _ = buffer.Unlock();
    context.Unmap(&target, 0);
  }
  win(unsafe { buffer.SetCurrentLength(length as u32) })?;
  Ok(buffer)
}
