// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction passes' pipelines, bindings and draw.

use super::*;
use screenwide_gpu::{RenderStages, Shader};

/// Where each record sits in the records buffer. Every pass reads its own
/// record through a dynamic offset, since all passes of a frame are recorded
/// before any of them runs; Direct3D 12 aligns those offsets to 256 bytes.
pub(super) const RECORD_STRIDE: u64 = 256;

/// A pass of `shader` writing `format`.
pub(super) fn pipeline(shader: &Shader, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
  shader.render_pipeline(&RenderStages {
    label: "Screenwide redaction pipeline",
    vertex: "vs_main",
    fragment: "fs_main",
    targets: &[Some(wgpu::ColorTargetState {
      format,
      blend: None,
      write_mask: wgpu::ColorWrites::ALL,
    })],
    primitive: Default::default(),
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
