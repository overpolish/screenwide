// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const SIZE: u32 = 8;

/// Draws one full-target quad of `kind` over a transparent target and
/// returns the centre pixel as BGRA bytes.
fn draw(gpu: &Gpu, kind: u32, constants: &RenderConstants) -> [u8; 4] {
  let shared = gpu.device();
  let device = &shared.device;
  let view = Size {
    width: f64::from(SIZE),
    height: f64::from(SIZE),
  };
  let mut vertices = Vec::new();
  renderer::add_quad(
    &mut vertices,
    view,
    Rect::from_xywh(0.0, 0.0, view.width, view.height),
    kind,
  );
  let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: std::mem::size_of_val(vertices.as_slice()) as u64,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  shared
    .queue
    .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));
  let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: size_of::<RenderConstants>() as u64,
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  shared
    .queue
    .write_buffer(&constants_buffer, 0, bytemuck::bytes_of(constants));
  let target = device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: SIZE,
      height: SIZE,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: crate::gpu::surface::FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  let bytes_per_row = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
  let readback = device.create_buffer(&wgpu::BufferDescriptor {
    label: None,
    size: u64::from(bytes_per_row * SIZE),
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let bindings = gpu.bindings(
    &constants_buffer,
    &gpu.placeholder,
    &gpu.placeholder,
    &gpu.placeholder,
    &gpu.placeholder,
  );
  let target_view = target.create_view(&Default::default());
  let mut encoder = device.create_command_encoder(&Default::default());
  {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &target_view,
        depth_slice: None,
        resolve_target: None,
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
    pass.set_pipeline(&gpu.pipeline);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.set_bind_group(0, &bindings, &[0]);
    pass.draw(0..vertices.len() as u32, 0..1);
  }
  encoder.copy_texture_to_buffer(
    target.as_image_copy(),
    wgpu::TexelCopyBufferInfo {
      buffer: &readback,
      layout: wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(bytes_per_row),
        rows_per_image: Some(SIZE),
      },
    },
    target.size(),
  );
  shared.queue.submit([encoder.finish()]);
  let slice = readback.slice(..);
  slice.map_async(wgpu::MapMode::Read, |result| {
    result.expect("the readback maps");
  });
  device
    .poll(wgpu::PollType::wait_indefinitely())
    .expect("the device finishes the frame");
  let mapped = slice.get_mapped_range().expect("the readback is mapped");
  let centre = (SIZE / 2 * bytes_per_row + SIZE / 2 * 4) as usize;
  [
    mapped[centre],
    mapped[centre + 1],
    mapped[centre + 2],
    mapped[centre + 3],
  ]
}

#[test]
fn the_osc_shader_blends_straight_colour_over_the_target() {
  let gpu = Gpu::new().expect("the region OSC pipeline builds on the shared device");
  let errors = gpu
    .device()
    .device
    .push_error_scope(wgpu::ErrorFilter::Validation);
  let mut constants = RenderConstants::new(false);
  constants.action_fills[0] = [0.2, 0.4, 0.6, 1.0];
  constants.overlay_shade = [1.0, 0.0, 0.0, 0.4];

  // An opaque control fill lands as itself.
  assert_eq!(draw(&gpu, 12, &constants), [153, 102, 51, 255]);
  // Straight source-over: a 40% red over transparent keeps 40% of the colour
  // and its alpha whole, so the target holds premultiplied colour.
  assert_eq!(draw(&gpu, 6, &constants), [0, 0, 102, 102]);
  assert!(pollster::block_on(errors.pop()).is_none());
}
