// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scaler's pipelines and the resources every thumbnail reuses.

use super::*;
use screenwide_gpu::{RenderStages, Shader};

impl Scaler {
  pub(in super::super) fn create() -> Result<Self, String> {
    let gpu = screenwide_gpu::shared()?;
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide camera thumbnail",
      &crate::shader_layouts::CAMERA_THUMBNAIL,
      SHADER,
      crate::precompiled_shaders::CAMERA_THUMBNAIL,
    );
    let pipeline = |fragment| {
      shader.render_pipeline(&RenderStages {
        label: "Screenwide camera thumbnail pipeline",
        vertex: "vs_main",
        fragment,
        targets: &[Some(wgpu::ColorTargetState {
          format: FORMAT,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
        primitive: Default::default(),
      })
    };
    Ok(Self {
      gpu: std::sync::Arc::clone(&gpu),
      bgra: pipeline("fs_bgra"),
      biplanar: pipeline("fs_biplanar"),
      layout: shader.bind_group_layout(0).clone(),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Screenwide camera thumbnail sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      uniforms: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide camera thumbnail uniforms"),
        size: size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      target: None,
      // The largest thumbnail the island asks for, allocated once.
      readback: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide camera thumbnail readback"),
        size: u64::from(ROW_PITCH) * MAX_HEIGHT as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
      }),
    })
  }
}
