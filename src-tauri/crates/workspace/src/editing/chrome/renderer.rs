// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Draws the selection chrome through the shared OSC shader into a
//! transparent drawable laid over the workspace. The platform makes the
//! drawable; every vertex shares one constants block, so the chrome is one
//! draw.

use screenwide_gpu::surface::{Frame, Surface};
use screenwide_osc::gpu::vertices::{self as osc_gpu, RenderConstants, Vertex};

pub struct ChromeRenderer {
  shared: &'static screenwide_gpu::Gpu,
  surface: Surface,
  pipeline: wgpu::RenderPipeline,
  /// The constants block, and transparent texels for every texture the
  /// shader declares, which the chrome's own quads never sample.
  bindings: wgpu::BindGroup,
  constants: wgpu::Buffer,
  vertex_buffer: wgpu::Buffer,
  vertex_capacity: usize,
}

impl ChromeRenderer {
  pub fn new(shared: &'static screenwide_gpu::Gpu, surface: Surface) -> Self {
    let device = &shared.device;
    let module = osc_gpu::shader_module(device);
    let layout = osc_gpu::bind_group_layout(device);
    let pipeline = osc_gpu::pipeline(device, &layout, &module);
    let constants = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide selection constants"),
      size: size_of::<RenderConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let placeholder = shared
      .texture_with_pixels(
        "Screenwide selection placeholder",
        (1, 1, 1),
        wgpu::TextureFormat::Bgra8Unorm,
        &[0; 4],
      )
      .create_view(&Default::default());
    let texture = |binding| wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::TextureView(&placeholder),
    };
    let linear_sampler = osc_gpu::sampler(device, wgpu::FilterMode::Linear);
    let point_sampler = osc_gpu::sampler(device, wgpu::FilterMode::Nearest);
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide selection bindings"),
      layout: &layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &constants,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<RenderConstants>() as u64),
          }),
        },
        texture(1),
        texture(2),
        texture(3),
        texture(4),
        texture(5),
        wgpu::BindGroupEntry {
          binding: 6,
          resource: wgpu::BindingResource::Sampler(&linear_sampler),
        },
        wgpu::BindGroupEntry {
          binding: 7,
          resource: wgpu::BindingResource::Sampler(&point_sampler),
        },
      ],
    });
    let vertex_capacity = 256;
    Self {
      shared,
      surface,
      pipeline,
      bindings,
      constants,
      vertex_buffer: vertex_buffer(shared, vertex_capacity),
      vertex_capacity,
    }
  }

  /// Gives the drawable's memory back while the editor is hidden.
  pub fn release_drawables(&mut self) {
    self.surface.resize(self.shared, (2, 2));
  }

  pub fn present(
    &mut self,
    size: (u32, u32),
    vertices: &[Vertex],
    constants: &RenderConstants,
  ) -> Result<(), String> {
    let gpu = self.shared;
    self.surface.resize(gpu, size);
    let Frame::Ready(frame) = self.surface.acquire(gpu)? else {
      return Ok(());
    };
    if !vertices.is_empty() {
      if vertices.len() > self.vertex_capacity {
        self.vertex_capacity = vertices.len().next_power_of_two().max(256);
        self.vertex_buffer = vertex_buffer(gpu, self.vertex_capacity);
      }
      gpu
        .queue
        .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
      gpu
        .queue
        .write_buffer(&self.constants, 0, bytemuck::bytes_of(constants));
    }
    let target = frame.texture.create_view(&Default::default());
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide selection overlay"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide selection overlay"),
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
      if !vertices.is_empty() {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.bindings, &[0]);
        pass.draw(0..vertices.len() as u32, 0..1);
      }
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.queue.present(frame);
    Ok(())
  }
}

fn vertex_buffer(gpu: &screenwide_gpu::Gpu, capacity: usize) -> wgpu::Buffer {
  gpu.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Screenwide selection vertices"),
    size: (capacity * size_of::<Vertex>()) as u64,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  })
}
