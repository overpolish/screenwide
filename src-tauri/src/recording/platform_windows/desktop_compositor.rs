// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GPU composition for Windows regions that cross display boundaries.
//!
//! Each display is captured on the recording's own Direct3D 11 device; its
//! latest frame is copied into a texture wgpu samples, the pieces are drawn
//! onto the shared canvas in wgpu, and the canvas is copied out to a texture
//! of its own for the encoder. Fences order each hand-over between the two
//! devices.

use std::{mem::size_of, time::Instant};

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
      D3D11_BIND_SHADER_RESOURCE, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

use crate::desktop_capture::{CapturePiece, CapturePlan, FrameSynchronizer};
use crate::gpu::{BridgedTexture, D3d11Bridge, Gpu};

use super::writer::Frame;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/desktop_compositor.wgsl"));
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

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

pub(super) struct DesktopFrameCoordinator {
  compositor: DesktopCompositor,
  /// When each display's latest frame arrived, once one has.
  latest: Vec<Option<Instant>>,
  pieces: Vec<CapturePiece>,
  synchronizer: FrameSynchronizer,
}

impl DesktopFrameCoordinator {
  /// Composes on `device`, the capture device, which has to be on wgpu's
  /// adapter.
  pub fn new(device: &ID3D11Device, plan: &CapturePlan) -> Result<Self, String> {
    Ok(Self {
      compositor: DesktopCompositor::new(device, plan.width, plan.height, plan.pieces.len())?,
      latest: vec![None; plan.pieces.len()],
      pieces: plan.pieces.clone(),
      synchronizer: FrameSynchronizer::new(plan.pieces.len())?,
    })
  }

  pub fn update(&mut self, source_index: usize, frame: Frame) -> Result<Option<Frame>, String> {
    let slot = self
      .latest
      .get_mut(source_index)
      .ok_or_else(|| "A frame arrived from an unknown desktop source".to_owned())?;
    // Taken now: the capture recycles its surface once this callback returns.
    self.compositor.take(source_index, &frame.texture)?;
    *slot = Some(frame.wall);
    let Some(tick) = self.synchronizer.update(source_index, frame.source_100ns)? else {
      return Ok(None);
    };
    let wall = self
      .latest
      .iter()
      .map(|wall| wall.expect("the synchronizer waits for every desktop source"))
      .max()
      .unwrap_or_else(Instant::now);
    Ok(Some(Frame {
      source_100ns: tick.output_ns,
      texture: self.compositor.compose(&self.pieces)?,
      wall,
    }))
  }
}

struct DesktopCompositor {
  gpu: &'static Gpu,
  device: ID3D11Device,
  bridge: D3d11Bridge,
  width: u32,
  height: u32,
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
  /// Every piece's constants, one per dynamic-offset stride.
  constants: wgpu::Buffer,
  stride: u64,
  /// Each display's latest frame, sized like its capture.
  sources: Vec<Option<BridgedTexture>>,
  /// The shared canvas the pieces are drawn onto.
  canvas: BridgedTexture,
}

mod pipeline;
#[cfg(test)]
mod tests;

impl DesktopCompositor {
  /// Copies a display's frame into the texture its piece is drawn from.
  fn take(&mut self, index: usize, frame: &ID3D11Texture2D) -> Result<(), String> {
    let mut description = D3D11_TEXTURE2D_DESC::default();
    unsafe { frame.GetDesc(&mut description) };
    let size = (description.Width, description.Height);
    let slot = self
      .sources
      .get_mut(index)
      .ok_or_else(|| "A frame arrived from an unknown desktop source".to_owned())?;
    if slot
      .as_ref()
      .is_none_or(|source| (source.texture.width(), source.texture.height()) != size)
    {
      *slot = Some(
        self
          .bridge
          .shared_texture(self.gpu, size, FORMAT, "desktop source")?,
      );
    }
    let source = slot.as_ref().expect("the desktop source was just made");
    let frame: ID3D11Resource = frame.cast().map_err(|error| error.to_string())?;
    let destination: ID3D11Resource = source.d3d11.cast().map_err(|error| error.to_string())?;
    unsafe { self.bridge.context.CopyResource(&destination, &frame) };
    self.bridge.d3d11_to_gpu(self.gpu)
  }

  /// Draws every piece onto the canvas and copies it out to a texture of
  /// its own: Media Foundation keeps a sample's texture until the encoder
  /// has read it, so the next frame may not draw over it.
  fn compose(&self, pieces: &[CapturePiece]) -> Result<ID3D11Texture2D, String> {
    if pieces.len() != self.sources.len() {
      return Err("Desktop frames no longer match the capture plan".to_owned());
    }
    let gpu = self.gpu;
    let mut constants = vec![0_u8; (self.stride as usize) * pieces.len()];
    let mut bindings = Vec::with_capacity(pieces.len());
    for (index, (piece, source)) in pieces.iter().zip(&self.sources).enumerate() {
      let source = source
        .as_ref()
        .ok_or_else(|| "A desktop source has no frame yet".to_owned())?;
      let values = PieceConstants::new(
        [self.width, self.height],
        [source.texture.width(), source.texture.height()],
        *piece,
      );
      let start = index * self.stride as usize;
      constants[start..start + size_of::<PieceConstants>()]
        .copy_from_slice(bytemuck::bytes_of(&values));
      bindings.push(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            resource: wgpu::BindingResource::TextureView(&source.view),
          },
          wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
        ],
      }));
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
          view: &self.canvas.view,
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
    gpu.queue.submit([encoder.finish()]);
    let description = D3D11_TEXTURE2D_DESC {
      Width: self.width,
      Height: self.height,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
      ..Default::default()
    };
    let mut texture = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&description, None, Some(&mut texture))
    }
    .map_err(|error| error.to_string())?;
    let texture = texture.ok_or_else(|| "Direct3D created no desktop frame".to_owned())?;
    let destination: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let canvas: ID3D11Resource = self
      .canvas
      .d3d11
      .cast()
      .map_err(|error| error.to_string())?;
    // The capture device waits for the canvas drawn above before copying it.
    self.bridge.gpu_to_d3d11(gpu)?;
    unsafe {
      self.bridge.context.CopyResource(&destination, &canvas);
      self.bridge.context.Flush();
    }
    Ok(texture)
  }
}
