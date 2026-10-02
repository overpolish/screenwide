// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Draws the pieces of a recording that crosses display boundaries onto one
//! canvas: each display's frame, scaled into its place, over black. The
//! platform brings the frames in as textures and hands the canvas on to its
//! encoder.

mod pipeline;
#[cfg(test)]
mod tests;

use std::mem::size_of;

use crate::desktop_capture::CapturePiece;
use crate::gpu::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/desktop_compositor.wgsl"));
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// The twin of `Piece` in `desktop_compositor.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
struct PieceConstants {
  output_size: [u32; 2],
  source_size: [u32; 2],
  source_origin: [u32; 2],
  source_extent: [u32; 2],
  destination_origin: [u32; 2],
  destination_extent: [u32; 2],
}

const _: () = assert!(size_of::<PieceConstants>().is_multiple_of(16));

impl PieceConstants {
  fn new(output: [u32; 2], source: [u32; 2], piece: CapturePiece) -> Self {
    Self {
      output_size: output,
      source_size: source,
      source_origin: [piece.source_pixels.x, piece.source_pixels.y],
      source_extent: [piece.source_pixels.width, piece.source_pixels.height],
      destination_origin: [piece.destination.x, piece.destination.y],
      destination_extent: [piece.destination.width, piece.destination.height],
    }
  }
}

/// One display's frame and where its piece of the canvas comes from.
pub(crate) struct PieceSource<'a> {
  pub(crate) piece: CapturePiece,
  pub(crate) texture: &'a wgpu::Texture,
}

pub(crate) struct DesktopCanvas {
  gpu: &'static Gpu,
  width: u32,
  height: u32,
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
  /// Every piece's constants, one per dynamic-offset stride.
  constants: wgpu::Buffer,
  stride: u64,
}

impl DesktopCanvas {
  /// Clears `canvas`, a `width` by `height` texture in [`FORMAT`], to black
  /// and draws every piece onto it. Returns the submission, for a caller
  /// that has to wait for it.
  pub(crate) fn draw(
    &self,
    canvas: &wgpu::TextureView,
    sources: &[PieceSource<'_>],
  ) -> Result<wgpu::SubmissionIndex, String> {
    if sources.len() as u64 * self.stride > self.constants.size() {
      return Err("Desktop frames no longer match the capture plan".to_owned());
    }
    let gpu = self.gpu;
    let mut constants = vec![0_u8; (self.stride as usize) * sources.len()];
    let mut bindings = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
      let values = PieceConstants::new(
        [self.width, self.height],
        [source.texture.width(), source.texture.height()],
        source.piece,
      );
      let start = index * self.stride as usize;
      constants[start..start + size_of::<PieceConstants>()]
        .copy_from_slice(bytemuck::bytes_of(&values));
      bindings.push(self.bindings(&source.texture.create_view(&Default::default())));
    }
    gpu.queue.write_buffer(&self.constants, 0, &constants);
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide desktop compositor"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide desktop compositor"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: canvas,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      for (index, bindings) in bindings.iter().enumerate() {
        pass.set_bind_group(0, bindings, &[(index as u64 * self.stride) as u32]);
        pass.draw(0..6, 0..1);
      }
    }
    Ok(gpu.queue.submit([encoder.finish()]))
  }

  fn bindings(&self, source: &wgpu::TextureView) -> wgpu::BindGroup {
    self
      .gpu
      .device
      .create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide desktop piece"),
        layout: &self.layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer: &self.constants,
              offset: 0,
              size: wgpu::BufferSize::new(size_of::<PieceConstants>() as u64),
            }),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(source),
          },
          wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
        ],
      })
  }
}
