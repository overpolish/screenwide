// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Constants;
use crate::gpu::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/audio_ribbon.wgsl"));

/// The ribbon's pipeline and constants, independent of where it presents.
pub(super) struct Painter {
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  constants: wgpu::Buffer,
}

impl Painter {
  pub(super) fn new(device: &wgpu::Device) -> Self {
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
          format: crate::gpu::surface::FORMAT,
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
    Self {
      pipeline,
      layout,
      constants,
    }
  }

  pub(super) fn bindings(
    &self,
    device: &wgpu::Device,
    levels: &wgpu::TextureView,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
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

  /// Clears `target` and draws the row into it.
  pub(super) fn encode(
    &self,
    gpu: &Gpu,
    levels: &wgpu::BindGroup,
    constants: Constants,
    target: &wgpu::TextureView,
  ) {
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide audio ribbon"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide audio ribbon"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, levels, &[]);
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
  }
}
