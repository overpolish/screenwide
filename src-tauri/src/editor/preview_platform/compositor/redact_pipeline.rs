// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction passes' pipelines, bindings and draw.

use super::*;
use crate::editor::annotations::redact::records::RedactRecord;

/// Where each record sits in the records buffer. Every pass reads its own
/// record through a dynamic offset, since all passes of a frame are recorded
/// before any of them runs; Direct3D 12 aligns those offsets to 256 bytes.
pub(super) const RECORD_STRIDE: u64 = 256;

/// A pass of `source` writing `format`.
pub(super) fn pipeline(
  device: &wgpu::Device,
  layout: &wgpu::PipelineLayout,
  source: &str,
  format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
  let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Screenwide redaction shader"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Screenwide redaction pipeline"),
    layout: Some(layout),
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
        format,
        blend: None,
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

pub(super) fn records_buffer(gpu: &Gpu, count: u64) -> wgpu::Buffer {
  gpu.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Screenwide redaction records"),
    size: count.max(1) * RECORD_STRIDE,
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  })
}

pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let texture = |binding| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: false },
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    },
    count: None,
  };
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide redaction bindings"),
    entries: &[
      wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: true,
          min_binding_size: wgpu::BufferSize::new(size_of::<RedactRecord>() as u64),
        },
        count: None,
      },
      texture(1),
      wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Storage { read_only: true },
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      },
      texture(3),
      texture(4),
    ],
  })
}

/// One triangle through `pipeline` into `target`, over the viewport
/// `[x, y, width, height]`. What lies outside the viewport is kept: a paint
/// pass leaves the copy's own pixels around its box, and a scratch target is
/// only ever read where this pass wrote.
pub(super) fn draw(
  encoder: &mut wgpu::CommandEncoder,
  target: &wgpu::TextureView,
  viewport: [f32; 4],
  pipeline: &wgpu::RenderPipeline,
  bindings: &wgpu::BindGroup,
  offset: u32,
) {
  // wgpu rejects an empty viewport; an empty box draws nothing anyway.
  if viewport[2] <= 0.0 || viewport[3] <= 0.0 {
    return;
  }
  let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide redaction pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view: target,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  pass.set_pipeline(pipeline);
  pass.set_bind_group(0, bindings, &[offset]);
  pass.set_viewport(viewport[0], viewport[1], viewport[2], viewport[3], 0.0, 1.0);
  pass.draw(0..3, 0..1);
}
