// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's pipeline and the resources every frame reuses.

use super::*;

impl DesktopCompositor {
  pub(super) fn new(
    capture: &ID3D11Device,
    width: u32,
    height: u32,
    pieces: usize,
  ) -> Result<Self, String> {
    let gpu = crate::gpu::shared()?;
    let bridge = D3d11Bridge::new(gpu, capture)?;
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide desktop compositor shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide desktop compositor bindings"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: wgpu::BufferSize::new(size_of::<PieceConstants>() as u64),
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide desktop compositor layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide desktop compositor pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: FORMAT,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    });
    let alignment = u64::from(device.limits().min_uniform_buffer_offset_alignment);
    let stride = (size_of::<PieceConstants>() as u64).next_multiple_of(alignment);
    let canvas = bridge.shared_texture(gpu, (width, height), FORMAT, "desktop canvas")?;
    Ok(Self {
      gpu,
      device: capture.clone(),
      bridge,
      width,
      height,
      layout,
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
      sources: (0..pieces).map(|_| None).collect(),
      canvas,
    })
  }
}
