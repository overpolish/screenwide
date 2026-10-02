// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::Graphics::Direct3D11::{
  D3D11_CPU_ACCESS_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SUBRESOURCE_DATA,
  D3D11_USAGE_STAGING,
};

use super::*;
use crate::desktop_capture::{DesktopRect, PixelRect};

fn piece(display_id: u32, destination_x: u32) -> CapturePiece {
  CapturePiece {
    display_id,
    source_pixels: PixelRect {
      x: 0,
      y: 0,
      width: 2,
      height: 2,
    },
    destination: PixelRect {
      x: destination_x,
      y: 0,
      width: 2,
      height: 2,
    },
  }
}

fn plan() -> CapturePlan {
  CapturePlan {
    desktop_region: DesktopRect {
      x: 0.0,
      y: 0.0,
      width: 4.0,
      height: 2.0,
    },
    width: 4,
    height: 2,
    output_scale: 1.0,
    pieces: vec![piece(1, 0), piece(2, 2)],
  }
}

fn description(width: u32, height: u32) -> D3D11_TEXTURE2D_DESC {
  D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
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
  }
}

/// A 2x2 display frame of one BGRA colour, as a capture delivers it.
fn frame(device: &ID3D11Device, source_100ns: i64, bgra: [u8; 4]) -> Frame {
  let pixels: Vec<u8> = (0..4).flat_map(|_| bgra).collect();
  let mut texture = None;
  unsafe {
    device.CreateTexture2D(
      &description(2, 2),
      Some(&D3D11_SUBRESOURCE_DATA {
        pSysMem: pixels.as_ptr().cast(),
        SysMemPitch: 8,
        SysMemSlicePitch: 0,
      }),
      Some(&mut texture),
    )
  }
  .unwrap();
  Frame {
    source_100ns,
    texture: texture.unwrap(),
    wall: Instant::now(),
  }
}

/// The first row of `texture`, BGRA.
fn first_row(device: &ID3D11Device, texture: &ID3D11Texture2D, width: u32) -> Vec<u8> {
  let mut staging = None;
  unsafe {
    device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        ..description(width, 2)
      },
      None,
      Some(&mut staging),
    )
  }
  .unwrap();
  let staging: ID3D11Resource = staging.unwrap().cast().unwrap();
  let source: ID3D11Resource = texture.cast().unwrap();
  let context = unsafe { device.GetImmediateContext() }.unwrap();
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  unsafe {
    context.CopyResource(&staging, &source);
    context
      .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
      .unwrap();
    let row = std::slice::from_raw_parts(mapped.pData.cast::<u8>(), width as usize * 4).to_vec();
    context.Unmap(&staging, 0);
    row
  }
}

#[test]
fn waits_for_every_source_then_composes_the_shared_canvas() {
  let device = super::super::capture::create_device().unwrap();
  let mut coordinator = DesktopFrameCoordinator::new(&device, &plan()).unwrap();
  let blue = [255, 0, 0, 255];
  let red = [0, 0, 255, 255];
  assert!(coordinator
    .update(0, frame(&device, 10, blue))
    .unwrap()
    .is_none());
  let composed = coordinator
    .update(1, frame(&device, 12, red))
    .unwrap()
    .unwrap();
  assert_eq!(composed.source_100ns, 12);
  let row = first_row(&device, &composed.texture, 4);
  assert_eq!(row[..4], blue);
  assert_eq!(row[12..16], red);
}
