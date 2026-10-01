// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::Graphics::{
  Direct3D11::{
    ID3D11RenderTargetView, ID3D11Resource, D3D11_CPU_ACCESS_READ, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
  },
  Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
};

use super::*;

const SIZE: (u32, u32) = (4, 4);

/// The first pixel Direct3D 11 sees of `shared`, as stored.
fn d3d11_pixel(layer: &D3d11Layer, shared: &SharedTexture) -> [u8; 4] {
  let mut staging = None;
  unsafe {
    layer.device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Width: SIZE.0,
        Height: SIZE.1,
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
      },
      None,
      Some(&mut staging),
    )
  }
  .unwrap();
  let staging: ID3D11Resource = staging.unwrap().cast().unwrap();
  layer
    .with(&[shared], |context, textures| unsafe {
      let source: ID3D11Resource = textures[0].cast().unwrap();
      context.CopyResource(&staging, &source);
      let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
      context
        .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
        .unwrap();
      let pixel = std::slice::from_raw_parts(mapped.pData.cast::<u8>(), 4);
      let pixel = [pixel[0], pixel[1], pixel[2], pixel[3]];
      context.Unmap(&staging, 0);
      pixel
    })
    .unwrap()
}

#[test]
fn a_shared_texture_carries_pixels_both_ways() {
  let gpu = super::super::shared().expect("the shared device opens");
  let layer = d3d11().expect("Direct3D 11 layers on the shared device");
  let shared = layer
    .shared_texture(gpu, SIZE, wgpu::TextureFormat::Bgra8Unorm, "test frame")
    .expect("the texture is shared");

  // Direct3D 11 draws, wgpu reads.
  layer
    .with(&[&shared], |context, textures| unsafe {
      let mut target: Option<ID3D11RenderTargetView> = None;
      layer
        .device
        .CreateRenderTargetView(&textures[0], None, Some(&mut target))
        .unwrap();
      context.ClearRenderTargetView(&target.unwrap(), &[1.0, 0.0, 0.0, 1.0]);
    })
    .unwrap();
  let pixels = gpu.read_texture(&shared.texture).unwrap();
  assert_eq!(pixels[..4], [0, 0, 255, 255]);

  // wgpu draws, Direct3D 11 reads.
  let mut encoder = gpu.device.create_command_encoder(&Default::default());
  encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: None,
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view: &shared.view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
        store: wgpu::StoreOp::Store,
      },
    })],
    depth_stencil_attachment: None,
    timestamp_writes: None,
    occlusion_query_set: None,
    multiview_mask: None,
  });
  gpu.queue.submit([encoder.finish()]);
  assert_eq!(d3d11_pixel(layer, &shared), [255, 0, 0, 255]);
}
