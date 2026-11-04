// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A frame drawn across a whole surface bigger than it: the frame in the
//! surface's top start corner, its last column and row repeated over the
//! rest. On Windows the picture on screen is resized at once and the frame
//! drawn at its new size reaches the screen a refresh or more later, so the
//! picture can show a little of the surface past the frame shown: there it
//! shows the frame's own edge, each row carried on, rather than what an older
//! frame left there.

use screenwide_gpu::surface::FORMAT;

const SHADER: &str = r#"
@group(0) @binding(0) var frame: texture_2d<f32>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let last = vec2<i32>(textureDimensions(frame)) - vec2<i32>(1);
  return textureLoad(frame, clamp(vec2<i32>(position.xy), vec2<i32>(0), last), 0);
}
"#;

pub(crate) struct EdgeFill {
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
}

impl EdgeFill {
  pub(crate) fn new(device: &wgpu::Device) -> Self {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide settings preview edge fill shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide settings preview edge fill bindings"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
          sample_type: wgpu::TextureSampleType::Float { filterable: false },
          view_dimension: wgpu::TextureViewDimension::D2,
          multisampled: false,
        },
        count: None,
      }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide settings preview edge fill layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide settings preview edge fill"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some("vertex_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some("fragment_main"),
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
    Self { layout, pipeline }
  }

  /// Draws `frame` over the whole of `target`.
  pub(crate) fn draw(
    &self,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    frame: &wgpu::Texture,
    target: &wgpu::Texture,
  ) {
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide settings preview edge fill frame"),
      layout: &self.layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::TextureView(&frame.create_view(&Default::default())),
      }],
    });
    let view = target.create_view(&Default::default());
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Screenwide settings preview edge fill"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
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
}
