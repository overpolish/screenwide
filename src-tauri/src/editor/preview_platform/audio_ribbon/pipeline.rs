// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::ribbon_layout::AUDIO_RIBBON;
use super::Constants;
use screenwide_gpu::{Gpu, RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/audio_ribbon.wgsl"));

/// The ribbon's pipeline and constants, independent of where it presents.
pub(super) struct Painter {
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  constants: wgpu::Buffer,
}

impl Painter {
  pub(super) fn new(device: &wgpu::Device) -> Self {
    let shader = Shader::new(
      device,
      "Screenwide audio ribbon",
      &AUDIO_RIBBON,
      SHADER,
      crate::precompiled_shaders::AUDIO_RIBBON,
    );
    // One draw over a cleared target, so it is written rather than blended.
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide audio ribbon pipeline",
      vertex: "vs_main",
      fragment: "fs_main",
      targets: &[Some(wgpu::ColorTargetState {
        format: screenwide_gpu::surface::FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    let constants = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide audio ribbon constants"),
      size: size_of::<Constants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    Self {
      pipeline,
      layout: shader.bind_group_layout(0).clone(),
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
