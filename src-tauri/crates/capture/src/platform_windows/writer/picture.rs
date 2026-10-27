// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's picture readings (`screenwide-activity`), taken from each
//! frame as it is written, as macOS takes them, so a recording's readings
//! are ready when it stops. A frame lives on the GPU here: when a reading
//! falls due, ten times a second, the GPU shrinks it by building its mip
//! chain, and only a level about 512 wide is read back, not the frame.

use screenwide_activity::{grey_from_bgra, PictureTap};
use windows::Win32::Graphics::Direct3D11::{
  ID3D11DeviceContext, ID3D11ShaderResourceView, D3D11_CPU_ACCESS_READ, D3D11_MAPPED_SUBRESOURCE,
  D3D11_MAP_READ, D3D11_RESOURCE_MISC_GENERATE_MIPS, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use super::*;

/// The widest mip level read back, in pixels.
const READ_WIDTH: u32 = 512;

/// What the GPU shrinks frames of one size with, kept while frames keep
/// that size.
pub(super) struct PictureShrink {
  level: u32,
  level_size: (u32, u32),
  mips: ID3D11Resource,
  size: (u32, u32),
  staging: ID3D11Resource,
  view: ID3D11ShaderResourceView,
}

impl Writer {
  /// The readings for the movie at `path`, or none if they cannot be
  /// written: they help the editor, and never stop a recording.
  pub(super) fn picture_tap(path: &std::path::Path) -> Option<PictureTap> {
    PictureTap::create(path)
      .inspect_err(|error| eprintln!("Not reading this recording's picture: {error}"))
      .ok()
  }

  pub(super) fn read_picture(&mut self, frame: &Frame, pts_ns: i64) {
    let Some(tap) = self.picture.as_mut() else {
      return;
    };
    let device = &self.config.device;
    let shrink = &mut self.picture_shrink;
    if let Err(error) = tap.frame(pts_ns, || grey(device, shrink, &frame.texture)) {
      eprintln!("Stopped reading this recording's picture: {error}");
      self.picture = None;
    }
  }

  pub(super) fn finish_picture(&mut self, end_ns: i64) {
    if let Some(tap) = self.picture.take() {
      if let Err(error) = tap.finish(end_ns) {
        eprintln!("Could not finish this recording's picture readings: {error}");
      }
    }
  }
}

/// The frame as grey cells, or none if it is not BGRA or the GPU refuses.
fn grey(
  device: &ID3D11Device,
  shrink: &mut Option<PictureShrink>,
  texture: &ID3D11Texture2D,
) -> Option<Vec<u8>> {
  let mut description = D3D11_TEXTURE2D_DESC::default();
  unsafe { texture.GetDesc(&mut description) };
  if description.Format != DXGI_FORMAT_B8G8R8A8_UNORM {
    return None;
  }
  let size = (description.Width, description.Height);
  if shrink.as_ref().is_none_or(|current| current.size != size) {
    *shrink = PictureShrink::new(device, &description).ok();
  }
  let shrink = shrink.as_ref()?;
  let context = unsafe { device.GetImmediateContext() }.ok()?;
  let source = texture.cast::<ID3D11Resource>().ok()?;
  unsafe {
    context.CopySubresourceRegion(&shrink.mips, 0, 0, 0, 0, &source, 0, None);
    context.GenerateMips(&shrink.view);
    context.CopySubresourceRegion(
      &shrink.staging,
      0,
      0,
      0,
      0,
      &shrink.mips,
      shrink.level,
      None,
    );
  }
  read_back(&context, shrink)
}

fn read_back(context: &ID3D11DeviceContext, shrink: &PictureShrink) -> Option<Vec<u8>> {
  let (width, height) = shrink.level_size;
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  unsafe {
    context
      .Map(&shrink.staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
      .ok()?;
    let stride = mapped.RowPitch as usize;
    let pixels = std::slice::from_raw_parts(mapped.pData.cast::<u8>(), stride * height as usize);
    let grey = grey_from_bgra(pixels, width as usize, height as usize, stride);
    context.Unmap(&shrink.staging, 0);
    Some(grey)
  }
}

impl PictureShrink {
  fn new(device: &ID3D11Device, frame: &D3D11_TEXTURE2D_DESC) -> Result<Self, String> {
    let size = (frame.Width, frame.Height);
    let mut level = 0;
    while (size.0 >> level) > READ_WIDTH {
      level += 1;
    }
    let level_size = ((size.0 >> level).max(1), (size.1 >> level).max(1));
    let mut mips = None;
    win(unsafe {
      device.CreateTexture2D(
        &D3D11_TEXTURE2D_DESC {
          ArraySize: 1,
          BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
          CPUAccessFlags: 0,
          MipLevels: level + 1,
          MiscFlags: D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32,
          Usage: D3D11_USAGE_DEFAULT,
          ..*frame
        },
        None,
        Some(&mut mips),
      )
    })?;
    let mips = mips.ok_or("Direct3D made no texture to shrink frames in")?;
    let mut view = None;
    win(unsafe { device.CreateShaderResourceView(&mips, None, Some(&mut view)) })?;
    let mut staging = None;
    win(unsafe {
      device.CreateTexture2D(
        &D3D11_TEXTURE2D_DESC {
          ArraySize: 1,
          BindFlags: 0,
          CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
          Height: level_size.1,
          MipLevels: 1,
          MiscFlags: 0,
          Usage: D3D11_USAGE_STAGING,
          Width: level_size.0,
          ..*frame
        },
        None,
        Some(&mut staging),
      )
    })?;
    Ok(Self {
      level,
      level_size,
      mips: win(mips.cast::<ID3D11Resource>())?,
      size,
      staging: win(
        staging
          .ok_or("Direct3D made no texture to read frames back from")?
          .cast::<ID3D11Resource>(),
      )?,
      view: view.ok_or("Direct3D made no view to shrink frames with")?,
    })
  }
}
