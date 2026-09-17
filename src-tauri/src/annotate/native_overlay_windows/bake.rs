// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing the annotations into a still that leaves the app as pixels.
//!
//! The clipboard has no layers, so a shot copied rather than opened carries
//! the arrows in its own pixels. They are drawn by the overlay's own pipeline,
//! offscreen, and composed over the still: the same shader draws the arrow
//! whether it is on the desktop, in the editor or on the clipboard.

use super::*;

use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11RenderTargetView, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
  D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};

use crate::editor::annotations::Annotation;
use crate::screenshots::CapturedImage;

/// The still with `annotations` drawn over it, in the still's own pixels.
///
/// The annotations arrive already placed in those pixels, so this is the
/// overlay's identity placement again, one drawn pixel per annotation pixel.
pub(crate) fn bake(
  image: &CapturedImage,
  annotations: &[Annotation],
) -> Result<CapturedImage, String> {
  if annotations.is_empty() || image.width == 0 || image.height == 0 {
    return Ok(image.clone());
  }
  let renderer = Renderer::new()?;
  let device = renderer.device().device();
  let target_texture = texture(device, image.width, image.height, false)?;
  let target_resource: ID3D11Resource = target_texture.cast().map_err(|e| e.to_string())?;
  let mut view: Option<ID3D11RenderTargetView> = None;
  unsafe { device.CreateRenderTargetView(&target_resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  let view = view.ok_or_else(|| "D3D11 created no annotate still target".to_owned())?;
  renderer.draw_arrows(
    &view,
    (image.width, image.height),
    &arrows::placed_arrows(annotations, (0.0, 0.0), (1.0, 1.0), None),
  )?;

  let staging = texture(device, image.width, image.height, true)?;
  let staging_resource: ID3D11Resource = staging.cast().map_err(|e| e.to_string())?;
  let context = renderer.device().context();
  let mut baked = image.clone();
  unsafe {
    context.CopyResource(&staging_resource, &target_resource);
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    context
      .Map(&staging_resource, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
      .map_err(|error| format!("The annotate still could not be read back: {error}"))?;
    compose(
      &mut baked,
      mapped.pData.cast::<u8>(),
      mapped.RowPitch as usize,
    );
    context.Unmap(&staging_resource, 0);
  }
  Ok(baked)
}

/// Source-over, in place. The drawn layer is premultiplied BGRA and the still
/// is opaque RGBA, so one multiply-add per channel is the whole blend.
///
/// # Safety
/// `drawn` must address `image.height` rows of `pitch` bytes, each holding at
/// least `image.width` BGRA pixels.
unsafe fn compose(image: &mut CapturedImage, drawn: *const u8, pitch: usize) {
  for y in 0..image.height as usize {
    let row = unsafe { drawn.add(y * pitch) };
    for x in 0..image.width as usize {
      let source = unsafe { std::slice::from_raw_parts(row.add(x * 4), 4) };
      let alpha = u32::from(source[3]);
      if alpha == 0 {
        continue;
      }
      let destination = &mut image.rgba[(y * image.width as usize + x) * 4..][..4];
      let over = |source: u8, destination: u8| {
        // The still is opaque, so rounding the remainder of the destination is
        // the only arithmetic the blend needs.
        let kept = u32::from(destination) * (255 - alpha) + 127;
        (u32::from(source) + kept / 255).min(255) as u8
      };
      destination[0] = over(source[2], destination[0]);
      destination[1] = over(source[1], destination[1]);
      destination[2] = over(source[0], destination[2]);
      destination[3] = 255;
    }
  }
}

fn texture(
  device: &ID3D11Device,
  width: u32,
  height: u32,
  readback: bool,
) -> Result<ID3D11Texture2D, String> {
  let mut texture = None;
  unsafe {
    device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
          Count: 1,
          Quality: 0,
        },
        Usage: if readback {
          D3D11_USAGE_STAGING
        } else {
          D3D11_USAGE_DEFAULT
        },
        BindFlags: if readback {
          0
        } else {
          D3D11_BIND_RENDER_TARGET.0 as u32
        },
        CPUAccessFlags: if readback {
          D3D11_CPU_ACCESS_READ.0 as u32
        } else {
          0
        },
        MiscFlags: 0,
      },
      None,
      Some(&mut texture),
    )
  }
  .map_err(|error| format!("The annotate still surface failed: {error}"))?;
  texture.ok_or_else(|| "D3D11 created no annotate still surface".to_owned())
}

#[cfg(test)]
#[path = "bake_tests.rs"]
mod tests;
