// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The crop tool's loupe, drawn into the workspace drawable over every layer
//! from the picture the layer it magnifies was drawn from.

use screenwide_gpu::{Gpu, RenderStages, Shader};
pub(super) use screenwide_osc::gpu::magnifier::NativeMagnifier;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/workspace_magnifier.wgsl"));

/// The twin of `Lens` in `lens.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LensConstants {
  box_rect: [f32; 4],
  source: [f32; 4],
  sample: [f32; 4],
  source_range: [f32; 4],
  flags: [u32; 4],
}

pub(super) struct Loupe {
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  constants: wgpu::Buffer,
  sampler: wgpu::Sampler,
}

impl Loupe {
  pub(super) fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide workspace loupe",
      &crate::shader_layouts::LOUPE,
      SHADER,
      crate::precompiled_shaders::LOUPE,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide workspace loupe",
      vertex: "vs_main",
      fragment: "fs_main",
      // The lens replaces what the layers drew under it.
      targets: &[Some(wgpu::ColorTargetState {
        format: screenwide_gpu::surface::FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    Self {
      pipeline,
      layout: shader.bind_group_layout(0).clone(),
      constants: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide workspace loupe constants"),
        size: size_of::<LensConstants>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Screenwide workspace loupe sampler"),
        ..Default::default()
      }),
    }
  }

  /// Draws `magnifier` into `target`, `size` drawable pixels, reading
  /// `picture`.
  pub(super) fn draw(
    &self,
    gpu: &Gpu,
    target: &wgpu::TextureView,
    size: (u32, u32),
    magnifier: &NativeMagnifier,
    picture: &wgpu::TextureView,
    picture_size: (u32, u32),
  ) {
    let [x, y] = magnifier.box_origin.map(i64::from);
    let [width, height] = magnifier.box_size.map(i64::from);
    let left = x.max(0);
    let top = y.max(0);
    let right = (x + width).min(i64::from(size.0));
    let bottom = (y + height).min(i64::from(size.1));
    if right <= left || bottom <= top || picture_size.0 == 0 || picture_size.1 == 0 {
      return;
    }
    let constants = LensConstants {
      box_rect: [x as f32, y as f32, width as f32, height as f32],
      source: [picture_size.0 as f32, picture_size.1 as f32, 0.0, 0.0],
      sample: [magnifier.sample[0], magnifier.sample[1], 0.0, 0.0],
      source_range: [
        magnifier.source_min[0],
        magnifier.source_min[1],
        magnifier.source_max[0],
        magnifier.source_max[1],
      ],
      flags: [magnifier.edges, 1, magnifier.light_mode, 0],
    };
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide workspace loupe bindings"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: self.constants.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(picture),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    });
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide workspace loupe"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide workspace loupe"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &bindings, &[]);
      pass.set_scissor_rect(
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
      );
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
  }
}
