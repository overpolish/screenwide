// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A texture's mip levels, each the average of the level above. A wide blur
//! read from the full picture needs many samples, and too few leave copies of
//! what it blurs; read from a level whose texels already average its width, a
//! few samples make it smooth.

use crate::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/mips.wgsl"));

/// The most levels a picture keeps: the seventh averages 64 by 64 texels,
/// past what any blur reads.
const LEVELS: u32 = 7;

/// How many levels a picture of `size` keeps, none smaller than a texel.
pub fn levels((width, height): (u32, u32)) -> u32 {
  LEVELS.min(32 - width.min(height).max(1).leading_zeros())
}

/// `texture`'s first level alone, which a pass draws into.
pub fn first_level(texture: &wgpu::Texture) -> wgpu::TextureView {
  level(texture, 0)
}

fn level(texture: &wgpu::Texture, level: u32) -> wgpu::TextureView {
  texture.create_view(&wgpu::TextureViewDescriptor {
    base_mip_level: level,
    mip_level_count: Some(1),
    ..Default::default()
  })
}

/// Makes the levels of textures in one format.
pub struct Mips {
  pipeline: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
}

impl Mips {
  pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide mip levels shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide mip levels"),
      layout: None,
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
        targets: &[Some(format.into())],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Screenwide mip levels sampler"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });
    Self { pipeline, sampler }
  }

  /// Every level of `texture` after the first made again from the first.
  pub fn fill(&self, gpu: &Gpu, texture: &wgpu::Texture) {
    let device = &gpu.device;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Screenwide mip levels"),
    });
    for at in 1..texture.mip_level_count() {
      let above = level(texture, at - 1);
      let target = level(texture, at);
      let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide mip level bindings"),
        layout: &self.pipeline.get_bind_group_layout(0),
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&above),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
        ],
      });
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide mip level"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
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
      pass.set_bind_group(0, &bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
  }
}
