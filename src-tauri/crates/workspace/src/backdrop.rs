// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The checkerboard the editor workspace's layers are drawn over. Every canvas
//! shows what lies past its edge over it, so what will not be exported can
//! still be seen, moved and deleted.

use screenwide_gpu::{Gpu, RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/workspace_backdrop.wgsl"));

/// A square's side in points, kept on screen at any zoom.
const SQUARE_POINTS: f64 = 8.0;

/// The squares' colours in the light and the dark appearance, as the drawable
/// holds them, before `OPACITY`.
const LIGHT: [[f32; 3]; 2] = [[1.0, 1.0, 1.0], [0.9, 0.9, 0.9]];
const DARK: [[f32; 3]; 2] = [[0.15, 0.15, 0.15], [0.2, 0.2, 0.2]];
/// How opaque the squares are: the window's glass still shows through them.
const OPACITY: f32 = 0.7;

/// How the native surface has the backdrop drawn: the drawable pixels in a
/// point, and whether the workspace has the light appearance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackdropStyle {
  pub scale: f64,
  pub light: bool,
}

/// The twin of `Backdrop` in `workspace_backdrop.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BackdropConstants {
  even: [f32; 4],
  odd: [f32; 4],
  square: [f32; 4],
}

pub struct Backdrop {
  pipeline: wgpu::RenderPipeline,
  constants: wgpu::Buffer,
  bindings: wgpu::BindGroup,
}

impl Backdrop {
  pub fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide workspace backdrop",
      &crate::shader_layouts::BACKDROP,
      SHADER,
      crate::precompiled_shaders::BACKDROP,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide workspace backdrop",
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
      label: Some("Screenwide workspace backdrop constants"),
      size: size_of::<BackdropConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide workspace backdrop bindings"),
      layout: shader.bind_group_layout(0),
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: constants.as_entire_binding(),
      }],
    });
    Self {
      pipeline,
      constants,
      bindings,
    }
  }

  /// Fills `target` with the checkerboard `style` asks for.
  pub fn draw(&self, gpu: &Gpu, target: &wgpu::TextureView, style: BackdropStyle) {
    // The drawable holds premultiplied colour.
    let [even, odd] = (if style.light { LIGHT } else { DARK })
      .map(|[r, g, b]| [r * OPACITY, g * OPACITY, b * OPACITY, OPACITY]);
    let square = (SQUARE_POINTS * style.scale.max(1.0)).round() as f32;
    let constants = BackdropConstants {
      even,
      odd,
      square: [square, 0.0, 0.0, 0.0],
    };
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide workspace backdrop"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide workspace backdrop"),
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
      pass.set_bind_group(0, &self.bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
  }
}
