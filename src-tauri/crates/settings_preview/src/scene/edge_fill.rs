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
use screenwide_gpu::{RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/edge_fill.wgsl"));

pub(crate) struct EdgeFill {
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
}

impl EdgeFill {
  pub(crate) fn new(device: &wgpu::Device) -> Self {
    let shader = Shader::new(
      device,
      "Screenwide settings preview edge fill",
      &crate::shader_layouts::EDGE_FILL,
      SHADER,
      crate::precompiled_shaders::EDGE_FILL,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide settings preview edge fill",
      vertex: "vertex_main",
      fragment: "fragment_main",
      targets: &[Some(wgpu::ColorTargetState {
        format: FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    Self {
      layout: shader.bind_group_layout(0).clone(),
      pipeline,
    }
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
