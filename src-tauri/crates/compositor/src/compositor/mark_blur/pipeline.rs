// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mark blur's GPU objects: the Gaussian pass's bindings and pipeline,
//! the layer's textures, and the one triangle every pass draws.

use super::{Gpu, LAYER_FORMAT};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/annotation_blur.wgsl"));

/// The Gaussian pass's bindings: its axis, and the two layers it reads.
pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty,
    count: None,
  };
  let layer = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::D2,
    multisampled: false,
  };
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide mark blur bindings"),
    entries: &[
      entry(
        0,
        wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
      ),
      entry(1, layer),
      entry(2, layer),
    ],
  })
}

/// The Gaussian pass along one axis of both layers.
pub(super) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
  let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Screenwide mark blur shader"),
    source: wgpu::ShaderSource::Wgsl(SHADER.into()),
  });
  let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Screenwide mark blur layout"),
    bind_group_layouts: &[Some(layout)],
    immediate_size: 0,
  });
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide mark blur pipeline"),
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
      targets: &[Some(LAYER_FORMAT.into()), Some(LAYER_FORMAT.into())],
    }),
    primitive: Default::default(),
    depth_stencil: None,
    multisample: Default::default(),
    multiview_mask: None,
    cache: None,
  })
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
