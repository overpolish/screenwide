// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// The format every RGBA picture the overlay draws is uploaded in.
pub(super) const RGBA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(in crate::app_windows::screenshot_region::region_osc) fn upload_rgba(
  gpu: &screenwide_gpu::Gpu,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> wgpu::TextureView {
  upload_texture(gpu, rgba, width, height, RGBA, 4)
}

/// An RGBA picture as the first of its mip levels, the rest left for
/// `screenwide_gpu::mips::Mips` to make.
pub(super) fn upload_rgba_with_levels(
  gpu: &screenwide_gpu::Gpu,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> wgpu::Texture {
  let levels = screenwide_gpu::mips::levels((width, height));
  create_texture(gpu, rgba, width, height, RGBA, 4, levels)
}

fn upload_texture(
  gpu: &screenwide_gpu::Gpu,
  pixels: &[u8],
  width: u32,
  height: u32,
  format: wgpu::TextureFormat,
  bytes_per_pixel: u32,
) -> wgpu::TextureView {
  create_texture(gpu, pixels, width, height, format, bytes_per_pixel, 1)
    .create_view(&Default::default())
}

fn create_texture(
  gpu: &screenwide_gpu::Gpu,
  pixels: &[u8],
  width: u32,
  height: u32,
  format: wgpu::TextureFormat,
  bytes_per_pixel: u32,
  levels: u32,
) -> wgpu::Texture {
  let size = wgpu::Extent3d {
    width,
    height,
    depth_or_array_layers: 1,
  };
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide region OSC texture"),
    size,
    mip_level_count: levels,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format,
    // Levels after the first are drawn into.
    usage: wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::COPY_DST
      | if levels > 1 {
        wgpu::TextureUsages::RENDER_ATTACHMENT
      } else {
        wgpu::TextureUsages::empty()
      },
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
  texture
}
