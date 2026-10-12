// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A canvas variant's shader modules, from what the build compiled or from
//! WGSL, and the render pipeline built from them for each target.

use super::canvas_pipelines::Target;
use super::canvas_precompiled;
use super::canvas_variants::{source, KindMask};
use super::{mark_blur::LAYER_FORMAT, FORMAT};

/// A variant's stages. Compiled from WGSL, both are the one module; the
/// build compiles each stage to a module of its own.
pub(super) struct CanvasModules {
  pub(super) vertex: wgpu::ShaderModule,
  pub(super) fragment: wgpu::ShaderModule,
}

/// The modules drawing `kinds`: the build's where it compiled them and
/// `precompiled` allows, otherwise compiled from WGSL.
pub(super) fn modules(device: &wgpu::Device, kinds: KindMask, precompiled: bool) -> CanvasModules {
  precompiled
    .then(|| canvas_precompiled::modules(device, kinds))
    .flatten()
    .unwrap_or_else(|| {
      let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Screenwide preview shader"),
        source: wgpu::ShaderSource::Wgsl(source(kinds).into()),
      });
      CanvasModules {
        vertex: module.clone(),
        fragment: module,
      }
    })
}

pub(super) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::PipelineLayout,
  modules: &CanvasModules,
  target: Target,
) -> wgpu::RenderPipeline {
  let format = match target {
    Target::Canvas | Target::Layer => FORMAT,
    Target::Delta => LAYER_FORMAT,
  };
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
      module: &modules.vertex,
      entry_point: Some("vs_main"),
      compilation_options: Default::default(),
      buffers: &[],
    },
    fragment: Some(wgpu::FragmentState {
      module: &modules.fragment,
      entry_point: Some("fs_main"),
      compilation_options: Default::default(),
      targets: &[Some(wgpu::ColorTargetState {
        format,
        blend: matches!(target, Target::Layer).then_some(wgpu::BlendState {
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
