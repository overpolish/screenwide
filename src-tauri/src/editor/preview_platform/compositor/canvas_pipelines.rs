// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas pass's pipelines: a whole canvas, or a screenshot layer blended
//! over the layers under it, each in full or lean, and the pass that draws
//! what the annotations under a blurring spotlight add, for
//! [`super::mark_blur`]. A lean pipeline's module is the canvas shader with
//! `annotations_drawn` false, which lets the compiler drop every annotation
//! pass for a draw showing none. Every pipeline but the two full ones is made
//! the first time a draw asks, so a compositor that never needs one never
//! compiles it.

use std::sync::OnceLock;

use super::{mark_blur::LAYER_FORMAT, FORMAT, SHADER};

const DRAWN: &str = "const annotations_drawn: bool = true;";
const LEFT_OUT: &str = "const annotations_drawn: bool = false;";

pub(super) struct CanvasPipelines {
  layout: wgpu::PipelineLayout,
  module: wgpu::ShaderModule,
  /// The canvas, then the layer.
  full: [wgpu::RenderPipeline; 2],
  lean: [OnceLock<wgpu::RenderPipeline>; 2],
  lean_module: OnceLock<wgpu::ShaderModule>,
  delta: OnceLock<wgpu::RenderPipeline>,
}

impl CanvasPipelines {
  pub(super) fn new(
    device: &wgpu::Device,
    bindings: &wgpu::BindGroupLayout,
    module: &wgpu::ShaderModule,
  ) -> Self {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide preview layout"),
      bind_group_layouts: &[Some(bindings)],
      immediate_size: 0,
    });
    let full = [false, true].map(|layer| pipeline(device, &layout, module, FORMAT, layer));
    Self {
      layout,
      module: module.clone(),
      full,
      lean: [OnceLock::new(), OnceLock::new()],
      lean_module: OnceLock::new(),
      delta: OnceLock::new(),
    }
  }

  /// The pipeline for a whole canvas or a `layer`, `lean` where the draw
  /// shows no annotation.
  pub(super) fn get(
    &self,
    device: &wgpu::Device,
    layer: bool,
    lean: bool,
  ) -> &wgpu::RenderPipeline {
    let slot = usize::from(layer);
    if !lean {
      return &self.full[slot];
    }
    self.lean[slot].get_or_init(|| {
      let module = self.lean_module.get_or_init(|| {
        debug_assert!(
          SHADER.contains(DRAWN),
          "the canvas shader declares `annotations_drawn`"
        );
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
          label: Some("Screenwide preview shader, no annotations"),
          source: wgpu::ShaderSource::Wgsl(SHADER.replacen(DRAWN, LEFT_OUT, 1).into()),
        })
      });
      pipeline(device, &self.layout, module, FORMAT, layer)
    })
  }

  /// The pipeline that draws what the annotations under a blurring
  /// spotlight add to the canvas into the mark blur's layer. It is the full
  /// canvas shader, so the compiled shader the canvas already has is reused.
  pub(super) fn delta(&self, device: &wgpu::Device) -> &wgpu::RenderPipeline {
    self
      .delta
      .get_or_init(|| pipeline(device, &self.layout, &self.module, LAYER_FORMAT, false))
  }
}

fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::PipelineLayout,
  module: &wgpu::ShaderModule,
  format: wgpu::TextureFormat,
  layer: bool,
) -> wgpu::RenderPipeline {
  // A layer's canvas is premultiplied, so it lays over what is under it as
  // `source + destination * (1 - source alpha)`.
  let over = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
  };
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide preview pipeline"),
    layout: Some(layout),
    vertex: wgpu::VertexState {
      module,
      entry_point: Some("vs_main"),
      compilation_options: Default::default(),
      buffers: &[],
    },
    fragment: Some(wgpu::FragmentState {
      module,
      entry_point: Some("fs_main"),
      compilation_options: Default::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format,
        blend: layer.then_some(wgpu::BlendState {
          color: over,
          alpha: over,
        }),
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }),
    primitive: Default::default(),
    depth_stencil: None,
    multisample: Default::default(),
    multiview_mask: None,
    cache: None,
  })
}
