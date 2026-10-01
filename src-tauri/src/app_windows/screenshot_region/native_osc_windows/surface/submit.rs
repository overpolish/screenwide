// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  /// Releases session-sized presentation buffers while retaining captured
  /// textures and interaction state for the next visible session.
  pub(super) fn release_drawables(&mut self) {
    if self.drawables_released {
      return;
    }
    self.drawables_released = true;
    self.vertex_buffer = None;
    self.vertex_capacity = 0;
    self.constants_buffer = None;
    self.constants_capacity = 0;
    self.vertices.clear();
    self.vertices.shrink_to_fit();
    self.chain.resize(self.gpu.device(), (2, 2));
  }

  pub(super) fn submit(
    &mut self,
    vertices: &[Vertex],
    segments: &[Segment],
    constants: &RenderConstants,
    size: (u32, u32),
  ) -> Result<(), String> {
    let gpu = Arc::clone(&self.gpu);
    let shared = gpu.device();
    self.chain.resize(shared, size);
    let Frame::Ready(frame) = self.chain.acquire(shared)? else {
      return Ok(());
    };
    self.write_vertices(vertices);
    let stride = constants_stride(&shared.device);
    self.write_constants(segments, constants, stride);
    let magnifier = self
      .magnifier_source
      .as_ref()
      .map_or(&gpu.placeholder, |source| &source.view);
    let snapshot = self
      .snapshot
      .as_ref()
      .map_or(&gpu.placeholder, |source| &source.view);
    // macOS puts a non-composited OCR snapshot in an opaque CALayer beneath
    // its transparent Metal layer. Windows folds both into this target, so
    // every presented snapshot - not only Ruler's composited one - must keep
    // opaque destination alpha as translucent shading is drawn over it.
    let pipeline = if opaque_snapshot_target(self.snapshot_presented, self.snapshot.is_some()) {
      &gpu.opaque_pipeline
    } else {
      &gpu.pipeline
    };
    let target = frame.texture.create_view(&Default::default());
    let mut encoder = shared
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide region OSC frame"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide region OSC pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
          depth_slice: None,
          resolve_target: None,
          // Flip-model back buffers are undefined after a present.
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
      if let (Some(vertex_buffer), Some(constants_buffer)) =
        (self.vertex_buffer.as_ref(), self.constants_buffer.as_ref())
      {
        pass.set_pipeline(pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        // One draw call per constants block. The base scene is a single
        // segment; each folded-in control adds one because its fill,
        // foreground and label texture are its own.
        for (index, segment) in segments.iter().enumerate() {
          if segment.count == 0 {
            continue;
          }
          let label = segment.label.as_ref().unwrap_or(&gpu.placeholder);
          let secondary = segment.secondary.as_ref().unwrap_or(&gpu.placeholder);
          let bindings = gpu.bindings(constants_buffer, label, secondary, snapshot, magnifier);
          pass.set_bind_group(0, &bindings, &[(index * stride) as u32]);
          pass.draw(segment.start..segment.start + segment.count, 0..1);
        }
      }
    }
    shared.queue.submit([encoder.finish()]);
    shared.queue.present(frame);
    Ok(())
  }

  fn write_vertices(&mut self, vertices: &[Vertex]) {
    if vertices.is_empty() {
      return;
    }
    let device = &self.gpu.device().device;
    if self.vertex_buffer.is_none() || self.vertex_capacity < vertices.len() {
      let capacity = vertices.len().next_power_of_two().max(512);
      self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide region OSC vertices"),
        size: (capacity * size_of::<Vertex>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.vertex_capacity = capacity;
    }
    if let Some(buffer) = &self.vertex_buffer {
      self
        .gpu
        .device()
        .queue
        .write_buffer(buffer, 0, bytemuck::cast_slice(vertices));
    }
  }

  /// Every segment's constants differ only in its fills and chrome, so the
  /// frame's block is copied once per segment at the uniform alignment.
  fn write_constants(&mut self, segments: &[Segment], constants: &RenderConstants, stride: usize) {
    if segments.is_empty() {
      return;
    }
    let device = &self.gpu.device().device;
    if self.constants_buffer.is_none() || self.constants_capacity < segments.len() {
      let capacity = segments.len().next_power_of_two().max(8);
      self.constants_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide region OSC constants"),
        size: (capacity * stride) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.constants_capacity = capacity;
    }
    let mut bytes = vec![0_u8; segments.len() * stride];
    for (block, segment) in bytes.chunks_exact_mut(stride).zip(segments) {
      let mut frame = *constants;
      frame.action_fills = segment.action_fills;
      frame.chrome = segment.chrome;
      frame.chrome_outline = segment.chrome_outline;
      block[..size_of::<RenderConstants>()].copy_from_slice(bytemuck::bytes_of(&frame));
    }
    if let Some(buffer) = &self.constants_buffer {
      self.gpu.device().queue.write_buffer(buffer, 0, &bytes);
    }
  }
}

/// `RenderConstants` rounded up to the device's dynamic-offset alignment.
fn constants_stride(device: &wgpu::Device) -> usize {
  let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
  size_of::<RenderConstants>().div_ceil(alignment) * alignment
}
