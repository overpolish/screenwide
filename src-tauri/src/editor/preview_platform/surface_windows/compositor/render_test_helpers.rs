// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::core::Interface;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::{
  Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1},
  Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
    D3D11_BOX, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
  },
  Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
};

pub(super) const OUTPUT: (u32, u32) = (1920, 1080);
pub(super) fn device() -> (ID3D11Device, ID3D11DeviceContext) {
  let mut device = None;
  let mut context = None;
  unsafe {
    D3D11CreateDevice(
      None,
      D3D_DRIVER_TYPE_HARDWARE,
      Default::default(),
      D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
      D3D11_SDK_VERSION,
      Some(&mut device),
      None,
      Some(&mut context),
    )
  }
  .expect("D3D11 hardware device");
  let device = device.unwrap();
  let adapter = unsafe {
    device
      .cast::<IDXGIDevice>()
      .expect("DXGI device")
      .GetAdapter()
      .expect("DXGI adapter")
      .GetDesc()
      .expect("DXGI adapter description")
      .Description
  };
  let end = adapter
    .iter()
    .position(|value| *value == 0)
    .unwrap_or(adapter.len());
  eprintln!(
    "D3D11 test adapter: {}",
    String::from_utf16_lossy(&adapter[..end])
  );
  (device, context.unwrap())
}

pub(super) fn target(device: &ID3D11Device) -> ID3D11Texture2D {
  target_size(device, OUTPUT)
}

pub(super) fn target_size(device: &ID3D11Device, size: (u32, u32)) -> ID3D11Texture2D {
  let mut target = None;
  unsafe {
    device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Width: size.0,
        Height: size.1,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
          Count: 1,
          Quality: 0,
        },
        BindFlags: windows::Win32::Graphics::Direct3D11::D3D11_BIND_RENDER_TARGET.0 as u32,
        ..Default::default()
      },
      None,
      Some(&mut target),
    )
  }
  .expect("D3D11 render target");
  target.unwrap()
}

pub(super) fn read_pixel(
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  target: &ID3D11Texture2D,
  x: u32,
  y: u32,
) -> [u8; 4] {
  let description = D3D11_TEXTURE2D_DESC {
    Width: 1,
    Height: 1,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_STAGING,
    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
    ..Default::default()
  };
  let mut staging = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut staging)) }.unwrap();
  let staging = staging.unwrap();
  let destination: ID3D11Resource = staging.cast().unwrap();
  let source: ID3D11Resource = target.cast().unwrap();
  let region = D3D11_BOX {
    left: x,
    top: y,
    right: x + 1,
    bottom: y + 1,
    front: 0,
    back: 1,
  };
  unsafe {
    context.CopySubresourceRegion(&destination, 0, 0, 0, 0, &source, 0, Some(&region));
    context.Flush();
  }
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  unsafe {
    context
      .Map(&destination, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
      .unwrap();
  }
  let pixel = unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), 4) };
  let result = [pixel[0], pixel[1], pixel[2], pixel[3]];
  unsafe { context.Unmap(&destination, 0) };
  result
}

pub(super) fn read_top_strip(
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  target: &ID3D11Texture2D,
) -> Vec<u8> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: OUTPUT.0,
    Height: OUTPUT.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_STAGING,
    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
    ..Default::default()
  };
  let mut staging = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut staging)) }
    .expect("D3D11 staging texture");
  let staging = staging.unwrap();
  let destination: ID3D11Resource = staging.cast().unwrap();
  let source: ID3D11Resource = target.cast().unwrap();
  unsafe { context.CopyResource(&destination, &source) };
  unsafe { context.Flush() };
  let mut mapped = Default::default();
  unsafe {
    context
      .Map(&destination, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
      .expect("map D3D11 staging texture")
  };
  let mut strip = vec![0; OUTPUT.0 as usize * 64 * 4];
  for row in 0..64 {
    unsafe {
      std::ptr::copy_nonoverlapping(
        mapped
          .pData
          .cast::<u8>()
          .add(row * mapped.RowPitch as usize),
        strip.as_mut_ptr().add(row * OUTPUT.0 as usize * 4),
        strip.len() / 64,
      );
    }
  }
  unsafe { context.Unmap(&destination, 0) };
  strip
}
