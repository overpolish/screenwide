// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focus blur's GPU objects: the bindings and pipelines that make the
//! chain's levels and gather from them, the chain itself, and the one
//! triangle every pass draws.

use super::super::FORMAT;
use crate::precompiled_shaders::{FOCUS_BLUR, FOCUS_COPY};
use crate::shader_layouts::{FOCUS_GATHER, FOCUS_LEVEL};
use screenwide_gpu::{Gpu, RenderStages, Shader};

const GATHER_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/focus_blur.wgsl"));
const COPY_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/focus_copy.wgsl"));

/// The bindings a level is made with, a level above and a sampler, and the
/// gather's, which adds its constants and its boxes.
pub(super) fn layouts(device: &wgpu::Device) -> [wgpu::BindGroupLayout; 2] {
  [
    screenwide_gpu::bind_group_layout(
      device,
      "Screenwide focus level bindings",
      FOCUS_LEVEL.groups[0],
    ),
    screenwide_gpu::bind_group_layout(
      device,
      "Screenwide focus gather bindings",
      FOCUS_GATHER.groups[0],
    ),
  ]
}

/// The pass that makes a level from the one above it, then the gather onto a
/// whole canvas and over a layer.
pub(super) fn pipelines(
  device: &wgpu::Device,
) -> (wgpu::RenderPipeline, [wgpu::RenderPipeline; 2]) {
  let copy = Shader::new(
    device,
    "Screenwide focus level",
    &FOCUS_LEVEL,
    COPY_SHADER,
    FOCUS_COPY,
  );
  let blur = Shader::new(
    device,
    "Screenwide focus blur",
    &FOCUS_GATHER,
    GATHER_SHADER,
    FOCUS_BLUR,
  );
  // A layer's frame is premultiplied, so it lays over what is under it as
  // `source + destination * (1 - source alpha)`, as the canvas pass's does.
  let over = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    operation: wgpu::BlendOperation::Add,
  };
  let pipeline = |shader: &Shader, fragment, blend| {
    shader.render_pipeline(&RenderStages {
      label: "Screenwide focus pipeline",
      vertex: "vs_main",
      fragment,
      targets: &[Some(wgpu::ColorTargetState {
        format: FORMAT,
        blend,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    })
  };
  (
    pipeline(&copy, "fs_downsample", None),
    [
      pipeline(&blur, "fs_main", None),
      pipeline(
        &blur,
        "fs_main",
        Some(wgpu::BlendState {
          color: over,
          alpha: over,
        }),
      ),
    ],
  )
}

/// One slot's chain of levels, the size of the target or, for a placed
/// canvas, a little past it (`FocusPlan::chain_size`).
pub(super) struct Chain {
  pub(super) size: (u32, u32),
  pub(super) texture: wgpu::Texture,
  /// Every level, which the gather reads.
  pub(super) view: wgpu::TextureView,
}

/// A chain `size` texels at its first level, down to the levels the widest
/// blur reads.
pub(super) fn chain(gpu: &Gpu, size: (u32, u32)) -> Chain {
  let size = (size.0.max(1), size.1.max(1));
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide focus chain"),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: screenwide_gpu::mips::levels(size),
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats: &[],
  });
  Chain {
    size,
    view: texture.create_view(&Default::default()),
    texture,
  }
}

/// One level of `texture`, which a pass draws into or reads.
pub(super) fn level(texture: &wgpu::Texture, at: u32) -> wgpu::TextureView {
  texture.create_view(&wgpu::TextureViewDescriptor {
    base_mip_level: at,
    mip_level_count: Some(1),
    ..Default::default()
  })
}

/// The one triangle over `view`, through `pipeline` with `bindings`, within a
/// viewport of `viewport`'s size where it is given.
pub(super) fn draw(
  encoder: &mut wgpu::CommandEncoder,
  view: &wgpu::TextureView,
  load: wgpu::LoadOp<wgpu::Color>,
  (pipeline, bindings): (&wgpu::RenderPipeline, &wgpu::BindGroup),
  viewport: Option<(u32, u32)>,
) {
  let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide focus pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load,
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  if let Some((width, height)) = viewport {
    pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  }
  pass.set_pipeline(pipeline);
  pass.set_bind_group(0, bindings, &[]);
  pass.draw(0..3, 0..1);
}
