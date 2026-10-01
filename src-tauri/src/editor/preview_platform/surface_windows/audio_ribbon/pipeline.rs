// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/audio_ribbon.wgsl"));

impl AudioRibbon {
  /// The ribbon's visual under `root`, hidden until it has levels. The
  /// caller commits the tree.
  pub(in crate::editor::preview_platform::surface_windows) fn new(
    shared: &'static crate::gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }.map_err(|e| e.to_string())?;
    unsafe {
      visual
        .SetOffsetX2(-100000.0)
        .and_then(|_| root.AddVisual(&visual, true, None::<&IDCompositionVisual>))
    }
    .map_err(|e| e.to_string())?;
    let surface = VisualSurface::new(shared, &visual)?;
    let device = &shared.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide audio ribbon shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide audio ribbon bindings"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
      ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide audio ribbon layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    // One draw over a cleared target, so it is written rather than blended.
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide audio ribbon pipeline"),
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
          format: crate::app_windows::overlay_surface::FORMAT,
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
    let constants = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide audio ribbon constants"),
      size: size_of::<Constants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    Ok(Self {
      shared,
      surface,
      pipeline,
      layout,
      constants,
      levels: None,
      visual,
      viewport: (2, 2),
      composition: composition.clone(),
      offset: (0.0, 0.0),
      scale: 1.0,
      neutral: 1.0,
      dirty: true,
      last_constants: None,
      points: 0,
      playhead: 0.0,
    })
  }

  pub(super) fn bindings(&self, levels: &wgpu::TextureView) -> wgpu::BindGroup {
    self
      .shared
      .device
      .create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide audio ribbon bindings"),
        layout: &self.layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: self.constants.as_entire_binding(),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(levels),
          },
        ],
      })
  }
}
