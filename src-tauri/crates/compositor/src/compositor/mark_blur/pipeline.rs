// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mark blur's GPU objects: the Gaussian pass's bindings and pipeline,
//! the layer's textures, and the one triangle every pass draws.

use super::{Gpu, LAYER_FORMAT};
use crate::precompiled_shaders::MARK_BLUR;
use screenwide_gpu::{RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/annotation_blur.wgsl"));

/// The Gaussian pass along one axis of both layers, and the layout of its
/// bindings: its axis, and the two layers it reads.
pub(super) fn pipeline(device: &wgpu::Device) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
  let shader = Shader::new(
    device,
    "Screenwide mark blur",
    &crate::shader_layouts::MARK_BLUR,
    SHADER,
    MARK_BLUR,
  );
  let pipeline = shader.render_pipeline(&RenderStages {
    label: "Screenwide mark blur pipeline",
    vertex: "vs_main",
    fragment: "fs_main",
    targets: &[Some(LAYER_FORMAT.into()), Some(LAYER_FORMAT.into())],
    primitive: Default::default(),
  });
  (pipeline, shader.bind_group_layout(0).clone())
}

/// A texture the layer is drawn into or blurred through, `size` texels.
pub(super) fn layer_texture(gpu: &Gpu, size: (u32, u32), label: &str) -> wgpu::TextureView {
  gpu
    .device
    .create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: LAYER_FORMAT,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    })
    .create_view(&Default::default())
}

/// One triangle over the whole of every texture in `targets`, through
/// `draw`, or nothing but clearing them where there is nothing to draw.
pub(super) fn full_pass(
  encoder: &mut wgpu::CommandEncoder,
  targets: &[&wgpu::TextureView],
  draw: Option<(&wgpu::RenderPipeline, &wgpu::BindGroup)>,
  label: &str,
) {
  let attachments: Vec<_> = targets
    .iter()
    .map(|view| {
      Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          store: wgpu::StoreOp::Store,
        },
      })
    })
    .collect();
  let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some(label),
    color_attachments: &attachments,
    ..Default::default()
  });
  if let Some((pipeline, bindings)) = draw {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bindings, &[]);
    pass.draw(0..3, 0..1);
  }
}
