// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The one OSC pipeline every macOS surface draws through, with the icon
//! atlas and a placeholder for texture slots a draw leaves empty.

use std::sync::{LazyLock, Mutex};

use super::super::pipeline::{
  bind_group_layout, pipeline, sampler, shader_module, RenderConstants, Vertex,
};
use crate::gpu::Gpu;

pub(super) struct Renderer {
  pub(super) gpu: &'static Gpu,
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  linear: wgpu::Sampler,
  point: wgpu::Sampler,
  pub(super) placeholder: wgpu::TextureView,
  icons: wgpu::TextureView,
}

pub(super) static RENDERER: LazyLock<Result<Mutex<Renderer>, String>> = LazyLock::new(|| {
  let gpu = crate::gpu::shared()?;
  let device = &gpu.device;
  let module = shader_module(device);
  let layout = bind_group_layout(device);
  let atlas = crate::osc::controls::icon_atlas();
  Ok(Mutex::new(Renderer {
    pipeline: pipeline(device, &layout, &module),
    layout,
    linear: sampler(device, wgpu::FilterMode::Linear),
    point: sampler(device, wgpu::FilterMode::Nearest),
    placeholder: gpu
      .texture_with_pixels(
        "Screenwide OSC placeholder",
        (1, 1, 1),
        wgpu::TextureFormat::Rgba8Unorm,
        &[0; 4],
      )
      .create_view(&Default::default()),
    icons: gpu
      .texture_with_pixels(
        "Screenwide OSC icons",
        (atlas.width, atlas.height, 1),
        wgpu::TextureFormat::R8Unorm,
        atlas.pixels(),
      )
      .create_view(&Default::default()),
    gpu,
  }))
});

impl Renderer {
  pub(super) fn draw(
    &self,
    target: &wgpu::Texture,
    clear: bool,
    vertices: &[Vertex],
    constants: &RenderConstants,
    [label, secondary, snapshot, source]: [wgpu::TextureView; 4],
  ) {
    let device = &self.gpu.device;
    let queue = &self.gpu.queue;
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide OSC constants"),
      size: size_of::<RenderConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    queue.write_buffer(&uniform, 0, bytemuck::bytes_of(constants));
    let vertex_buffer = (!vertices.is_empty()).then(|| {
      let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide OSC vertices"),
        size: std::mem::size_of_val(vertices) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      queue.write_buffer(&buffer, 0, bytemuck::cast_slice(vertices));
      buffer
    });
    let texture = |binding, view| wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::TextureView(view),
    };
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide OSC draw"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &uniform,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<RenderConstants>() as u64),
          }),
        },
        texture(1, &label),
        texture(2, &secondary),
        texture(3, &self.icons),
        texture(4, &snapshot),
        texture(5, &source),
        wgpu::BindGroupEntry {
          binding: 6,
          resource: wgpu::BindingResource::Sampler(&self.linear),
        },
        wgpu::BindGroupEntry {
          binding: 7,
          resource: wgpu::BindingResource::Sampler(&self.point),
        },
      ],
    });
    let view = target.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Screenwide OSC frame"),
    });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide OSC pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: if clear {
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
              wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      if let Some(buffer) = &vertex_buffer {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.set_bind_group(0, &bindings, &[0]);
        pass.draw(0..vertices.len() as u32, 0..1);
      }
    }
    queue.submit([encoder.finish()]);
  }
}
