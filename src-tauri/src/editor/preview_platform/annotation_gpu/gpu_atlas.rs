// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// An atlas's BGRA texture: the type atlas's, and the image atlas's.
pub(super) struct GpuAtlas {
  texture: wgpu::Texture,
  pub(super) view: wgpu::TextureView,
}

impl GpuAtlas {
  /// A texture for a fresh layout. Its contents start undefined, which is
  /// safe because every cell is drawn whole, margin included, before it is
  /// sampled, and the shader reads nowhere else.
  pub(super) fn new(gpu: &crate::gpu::Gpu, label: &str, size: (u32, u32)) -> Self {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Bgra8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    Self { texture, view }
  }

  pub(super) fn write(
    &self,
    gpu: &crate::gpu::Gpu,
    origin: (u32, u32),
    size: (u32, u32),
    pixels: &[u8],
  ) {
    gpu.queue.write_texture(
      wgpu::TexelCopyTextureInfo {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d {
          x: origin.0,
          y: origin.1,
          z: 0,
        },
        aspect: wgpu::TextureAspect::All,
      },
      pixels,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(size.0 * 4),
        rows_per_image: Some(size.1),
      },
      wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
    );
  }

  pub(super) fn size(&self) -> (u32, u32) {
    let size = self.texture.size();
    (size.width, size.height)
  }
}
