// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas pipeline and the resources every frame reuses.

use super::*;
use screenwide_gpu::{RenderStages, Shader};

impl DesktopCanvas {
  /// A `width` by `height` canvas for at most `pieces` pieces.
  pub(crate) fn new(gpu: &std::sync::Arc<Gpu>, width: u32, height: u32, pieces: usize) -> Self {
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide desktop compositor",
      &crate::shader_layouts::DESKTOP_COMPOSITOR,
      SHADER,
      crate::precompiled_shaders::DESKTOP_COMPOSITOR,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide desktop compositor pipeline",
      vertex: "vs_main",
      fragment: "fs_main",
      targets: &[Some(wgpu::ColorTargetState {
        format: FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    let alignment = u64::from(device.limits().min_uniform_buffer_offset_alignment);
    let stride = (size_of::<PieceConstants>() as u64).next_multiple_of(alignment);
    Self {
      gpu: std::sync::Arc::clone(gpu),
      width,
      height,
      layout: shader.bind_group_layout(0).clone(),
      pipeline,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Screenwide desktop compositor sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      constants: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide desktop compositor pieces"),
        size: stride * pieces.max(1) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      stride,
    }
  }
}
