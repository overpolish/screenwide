// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn upload_icons(gpu: &crate::gpu::Gpu) -> Result<wgpu::TextureView, String> {
  let atlas = unsafe { screenwide_osc_icon_atlas() };
  let expected = atlas.width as usize * atlas.height as usize;
  if atlas.pixels.is_null() || atlas.length < expected || expected == 0 {
    return Err("the shared OSC icon atlas is empty".to_owned());
  }
  let pixels = unsafe { std::slice::from_raw_parts(atlas.pixels, expected) };
  Ok(upload_texture(
    gpu,
    pixels,
    atlas.width,
    atlas.height,
    wgpu::TextureFormat::R8Unorm,
    1,
  ))
}

pub(in crate::app_windows::screenshot_region::native_osc_windows) fn upload_rgba(
  gpu: &crate::gpu::Gpu,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> wgpu::TextureView {
  upload_texture(gpu, rgba, width, height, wgpu::TextureFormat::Rgba8Unorm, 4)
}

fn upload_texture(
  gpu: &crate::gpu::Gpu,
  pixels: &[u8],
  width: u32,
  height: u32,
  format: wgpu::TextureFormat,
  bytes_per_pixel: u32,
) -> wgpu::TextureView {
  let size = wgpu::Extent3d {
    width,
    height,
    depth_or_array_layers: 1,
  };
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide region OSC texture"),
    size,
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format,
    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  gpu.queue.write_texture(
    texture.as_image_copy(),
    pixels,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(width * bytes_per_pixel),
      rows_per_image: Some(height),
    },
    size,
  );
  texture.create_view(&Default::default())
}
