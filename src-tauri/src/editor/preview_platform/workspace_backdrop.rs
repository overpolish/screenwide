// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The checkerboard the editor workspace's layers are drawn over. Every canvas
//! shows what lies past its edge over it, so what will not be exported can
//! still be seen, moved and deleted.

use screenwide_gpu::Gpu;

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
pub(super) struct BackdropStyle {
  pub(super) scale: f64,
  pub(super) light: bool,
}

/// The twin of `Backdrop` in `workspace_backdrop.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BackdropConstants {
  even: [f32; 4],
  odd: [f32; 4],
  square: [f32; 4],
}

pub(super) struct Backdrop {
  pipeline: wgpu::RenderPipeline,
  constants: wgpu::Buffer,
  bindings: wgpu::BindGroup,
}

impl Backdrop {
  pub(super) fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide workspace backdrop shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide workspace backdrop bindings"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide workspace backdrop layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide workspace backdrop"),
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
          format: screenwide_gpu::surface::FORMAT,
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
      label: Some("Screenwide workspace backdrop constants"),
      size: size_of::<BackdropConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide workspace backdrop bindings"),
      layout: &layout,
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
  pub(super) fn draw(&self, gpu: &Gpu, target: &wgpu::TextureView, style: BackdropStyle) {
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
