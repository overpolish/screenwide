// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A focus drawn live: what lies around its box, softened as the editor's
//! focus softens it. The overlay never sees the desktop under it, so the blur
//! is read from the display's underlay, the desktop captured when the focus
//! was drawn, through a mip chain made of it once, and the falloff past the
//! box is the editor's own (`focus_falloff.wgsl`). Near the box, where the
//! blur is under a pixel wide, the overlay draws nothing and the live desktop
//! shows.

use screenwide_compositor::annotation_gpu::{
  GpuFocusRegion, PreparedArrows, FOCUS_REACH_SHARE, FOCUS_WIDEST_SHARE,
};
use screenwide_gpu::mips::{levels, Mips};
use screenwide_gpu::Gpu;
use screenwide_imaging::CapturedImage;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// What the focus pass of a frame asks: the boxes as the shader reads them,
/// and how far and how wide their blur reaches on a target of `size`.
pub(super) struct FocusFrame {
  pub(super) regions: Vec<GpuFocusRegion>,
  pub(super) reach: f32,
  pub(super) widest: f32,
}

impl FocusFrame {
  /// The focus boxes `prepared` holds, in the display's layer pixels: the
  /// overlay's screens are never turned, so no box has a depth to measure.
  pub(super) fn of(prepared: &PreparedArrows, (width, height): (u32, u32)) -> Self {
    let shorter = width.min(height).max(1) as f32;
    Self {
      regions: prepared
        .focus
        .iter()
        .map(|region| GpuFocusRegion::new(region, 1.0))
        .collect(),
      reach: FOCUS_REACH_SHARE * shorter,
      widest: FOCUS_WIDEST_SHARE * shorter,
    }
  }
}

/// Makes the mip chains a focus's blur reads.
pub(in crate::annotate) struct FocusChains {
  mips: Mips,
  pub(super) sampler: wgpu::Sampler,
}

impl FocusChains {
  pub(super) fn new(device: &wgpu::Device) -> Self {
    Self {
      mips: Mips::new(device, FORMAT),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Screenwide annotate focus sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
    }
  }

  /// `image` uploaded with every level of its mip chain, for the blur to
  /// read however wide it grows.
  pub(in crate::annotate) fn chain(
    &self,
    gpu: &Gpu,
    image: &CapturedImage,
  ) -> Result<wgpu::TextureView, String> {
    if image.width == 0 || image.height == 0 {
      return Err("A focus's underlay has no pixels".to_owned());
    }
    let size = wgpu::Extent3d {
      width: image.width,
      height: image.height,
      depth_or_array_layers: 1,
    };
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide annotate focus chain"),
      size,
      mip_level_count: levels((image.width, image.height)),
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: FORMAT,
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    gpu.queue.write_texture(
      texture.as_image_copy(),
      &image.rgba,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(image.width * 4),
        rows_per_image: Some(image.height),
      },
      size,
    );
    self.mips.fill(gpu, &texture);
    Ok(texture.create_view(&Default::default()))
  }
}
