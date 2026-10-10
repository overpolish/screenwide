// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The textures the effects pass draws into and reads from: one set for each
//! slot it draws, the matte uploaded beside the frame, and the picture.

use crate::editor::annotations::pin::LumaFrame;
use crate::editor::preview_platform::compositor::{SourceTexture, FORMAT};
use crate::gpu::Gpu;

/// The effects' generation and the frame time a matte was uploaded for.
type Uploaded = (u64, u64);

/// The textures one slot draws into, made again only when its frame's size
/// changes, and the matte last uploaded for it.
pub(super) struct Slot {
  pub(super) size: (u32, u32),
  pub(super) out: SourceTexture,
  matte: Option<(wgpu::Texture, (u32, u32), Uploaded)>,
}

impl Slot {
  pub(super) fn new(gpu: &Gpu, size: (u32, u32)) -> Self {
    let texture = target(gpu, size, "Screenwide camera with effects");
    Self {
      size,
      out: SourceTexture {
        size,
        view: texture.create_view(&Default::default()),
        texture,
        #[cfg(target_os = "windows")]
        shared: None,
        picture: None,
      },
      matte: None,
    }
  }

  /// The matte `frame` of the effects `generation` on the GPU, uploaded only
  /// when it is not the frame the slot holds already. The time alone does
  /// not tell: a matte made again, with objects laid in, has frames at the
  /// same times as the one before.
  pub(super) fn matte(
    &mut self,
    gpu: &Gpu,
    frame: &LumaFrame,
    generation: u64,
  ) -> wgpu::TextureView {
    let size = (frame.width, frame.height);
    let at = (generation, frame.ms);
    match self.matte.as_mut() {
      Some((texture, held, uploaded)) if *held == size => {
        if *uploaded != at {
          write(gpu, texture, size, &frame.pixels, 1);
          *uploaded = at;
        }
        texture.create_view(&Default::default())
      }
      _ => {
        let texture = upload(
          gpu,
          wgpu::TextureFormat::R8Unorm,
          size,
          &frame.pixels,
          "matte",
        );
        let view = texture.create_view(&Default::default());
        self.matte = Some((texture, size, at));
        view
      }
    }
  }
}

fn target(gpu: &Gpu, size: (u32, u32), label: &str) -> wgpu::Texture {
  gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some(label),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::RENDER_ATTACHMENT
      | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  })
}

/// What one pass of `camera_effects.wgsl` is told.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Uniform {
  pub(super) mode: [u32; 4],
  pub(super) gain: [f32; 4],
  pub(super) offset: [f32; 4],
  pub(super) picture: [f32; 4],
  pub(super) target_size: [f32; 4],
}

/// The part of `picture` a frame `size` covers when the picture fills it,
/// cut at its middle, as a scale and offset of the frame's coordinates.
pub(super) fn cover(picture: &::image::RgbaImage, size: (u32, u32)) -> [f32; 4] {
  let picture_aspect = picture.width() as f32 / picture.height().max(1) as f32;
  let frame_aspect = size.0 as f32 / size.1.max(1) as f32;
  if picture_aspect > frame_aspect {
    let scale = frame_aspect / picture_aspect;
    [scale, 1.0, (1.0 - scale) / 2.0, 0.0]
  } else {
    let scale = picture_aspect / frame_aspect;
    [1.0, scale, 0.0, (1.0 - scale) / 2.0]
  }
}

pub(super) fn upload(
  gpu: &Gpu,
  format: wgpu::TextureFormat,
  size: (u32, u32),
  pixels: &[u8],
  label: &str,
) -> wgpu::Texture {
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
    format,
    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  let bytes_per_pixel = format.block_copy_size(None).unwrap_or(4);
  write(gpu, &texture, size, pixels, bytes_per_pixel);
  texture
}

fn write(
  gpu: &Gpu,
  texture: &wgpu::Texture,
  size: (u32, u32),
  pixels: &[u8],
  bytes_per_pixel: u32,
) {
  gpu.queue.write_texture(
    texture.as_image_copy(),
    pixels,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(size.0 * bytes_per_pixel),
      rows_per_image: Some(size.1),
    },
    wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
  );
}

/// What every pass binds: its source, the matte and what is behind as
/// textures, the sampler, and the uniform.
pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let texture = |binding| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: true },
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    },
    count: None,
  };
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide camera effects bindings"),
    entries: &[
      texture(0),
      texture(1),
      texture(2),
      wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
      },
      wgpu::BindGroupLayoutEntry {
        binding: 4,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      },
    ],
  })
}
