// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction passes' pipelines, bindings and draw.

use super::*;
use screenwide_annotations::redact::records::RedactRecord;
use screenwide_gpu::{RenderStages, Shader};

/// Where each record sits in the records buffer. Every pass reads its own
/// record through a dynamic offset, since all passes of a frame are recorded
/// before any of them runs; Direct3D 12 aligns those offsets to 256 bytes.
pub(super) const RECORD_STRIDE: u64 = 256;

/// One redaction as its passes read it: the record, then the window it is
/// applied in. The twin of `Redaction` in `redact.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct GpuRedaction {
  pub(super) record: RedactRecord,
  /// Where the texture the passes read and draw over starts in the picture,
  /// then where the rows a blur's first pass writes start, in source pixels.
  pub(super) window: [u32; 4],
  /// How many of the picture's pixels a texel spans each way; the rest are
  /// spare.
  pub(super) scale: [u32; 4],
}

const _: () = assert!(size_of::<GpuRedaction>() as u64 <= RECORD_STRIDE);

/// One redaction over the window a texture holds of the picture: what its
/// passes read, the texels of the window its paint pass covers, x, y, width
/// and height, and how many texels its blur's rows take each way.
pub(super) struct WindowedPass {
  pub(super) redaction: GpuRedaction,
  pub(super) viewport: [f32; 4],
  pub(super) rows: (u32, u32),
}

/// The passes `records` take over a `size` texture holding `window` of the
/// picture, or all of it: none for a box the window holds nothing of.
pub(super) fn windowed_passes(
  records: &[RedactRecord],
  window: Option<SourceWindow>,
  size: (u32, u32),
) -> Vec<WindowedPass> {
  let (origin, span) = window.map_or(((0, 0), 1), |window| (window.origin, window.scale.max(1)));
  records
    .iter()
    .filter_map(|record| {
      let [x0, y0, x1, y1] = record.bounds;
      // The texels the box reaches into, all of each one it touches.
      let low = [x0.max(origin.0) - origin.0, y0.max(origin.1) - origin.1].map(|at| at / span);
      let high = [
        x1.min(origin.0 + size.0 * span).saturating_sub(origin.0),
        y1.min(origin.1 + size.1 * span).saturating_sub(origin.1),
      ]
      .map(|at| at.div_ceil(span));
      if low[0] >= high[0] || low[1] >= high[1] {
        return None;
      }
      let rows = (high[0] - low[0], high[1] - low[1]);
      Some(WindowedPass {
        redaction: GpuRedaction {
          record: *record,
          window: [
            origin.0,
            origin.1,
            origin.0 + low[0] * span,
            origin.1 + low[1] * span,
          ],
          scale: [span, 0, 0, 0],
        },
        viewport: [low[0] as f32, low[1] as f32, rows.0 as f32, rows.1 as f32],
        rows,
      })
    })
    .collect()
}

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

/// The bindings a pass reads `records` and `zones` through, from `source`
/// with the `cells` and `rows` an earlier pass left.
pub(super) fn bind_group(
  gpu: &Gpu,
  layout: &wgpu::BindGroupLayout,
  records: &wgpu::Buffer,
  zones: &wgpu::Buffer,
  [source, cells, rows]: [&wgpu::TextureView; 3],
) -> wgpu::BindGroup {
  let texture = |binding, view| wgpu::BindGroupEntry {
    binding,
    resource: wgpu::BindingResource::TextureView(view),
  };
  gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Screenwide redaction bindings"),
    layout,
    entries: &[
      wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
          buffer: records,
          offset: 0,
          size: wgpu::BufferSize::new(size_of::<GpuRedaction>() as u64),
        }),
      },
      texture(1, source),
      wgpu::BindGroupEntry {
        binding: 2,
        resource: zones.as_entire_binding(),
      },
      texture(3, cells),
      texture(4, rows),
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
