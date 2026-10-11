// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The picture an exported frame's screen is decoded into for the canvas
//! to sample.

use crate::editor::preview_platform::compositor::{SourceTexture, FORMAT};
use crate::gpu::Gpu;

/// A screen picture `size` large.
pub(super) fn canvas_source(gpu: &Gpu, size: (u32, u32)) -> SourceTexture {
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide export source"),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    // Redactions copy it before they draw over the copy.
    usage: wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::RENDER_ATTACHMENT
      | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  SourceTexture {
    size,
    view: texture.create_view(&Default::default()),
    texture,
    picture: None,
    enhanced: false,
    cut_out: false,
  }
}
