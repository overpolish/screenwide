// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording bar shows a 48 × 27 CSS-pixel camera confidence thumbnail,
//! and a 96 × 54 backing image keeps it crisp on Retina displays. Scaling on
//! the CPU meant a full-frame format conversion plus a ColorSync colour match
//! per frame, so the camera frame is wrapped as textures in place and sampled
//! straight down to the thumbnail on the GPU.

use std::sync::mpsc;

use cidre::cv;

use super::{MAX_HEIGHT, MAX_WIDTH};
use crate::gpu::macos::buffer_plane_texture;
use crate::gpu::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/camera_thumbnail.wgsl"));
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// A thumbnail row in the readback buffer, padded to the copy alignment.
const ROW_PITCH: u32 = (MAX_WIDTH as u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

/// The twin of `Thumbnail` in `camera_thumbnail.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
  size: [u32; 2],
  full_range: u32,
  spare: u32,
}

pub(super) struct Scaler {
  gpu: &'static Gpu,
  layout: wgpu::BindGroupLayout,
  bgra: wgpu::RenderPipeline,
  biplanar: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
  uniforms: wgpu::Buffer,
  /// The thumbnail drawn last, remade when the camera's size changes.
  target: Option<wgpu::Texture>,
  readback: wgpu::Buffer,
}

mod pipeline;

impl Scaler {
  /// Scales `buffer` into `out`, `width` by `height` RGBA, and waits for it.
  /// False for a pixel format the camera is never asked for, or a size past
  /// the recording bar's budget.
  pub(super) fn thumbnail(
    &mut self,
    buffer: &cv::PixelBuf,
    width: u16,
    height: u16,
    out: &mut [u8],
  ) -> bool {
    let (width, height) = (u32::from(width), u32::from(height));
    let row = width as usize * 4;
    if width == 0
      || height == 0
      || width as usize > MAX_WIDTH
      || height as usize > MAX_HEIGHT
      || out.len() < row * height as usize
    {
      return false;
    }
    match self.draw(buffer, width, height) {
      Ok(true) => {}
      Ok(false) => return false,
      Err(error) => {
        eprintln!("The camera thumbnail could not be drawn: {error}");
        return false;
      }
    }
    let slice = self
      .readback
      .slice(..u64::from(ROW_PITCH) * u64::from(height));
    let (sender, receiver) = mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
      let _ = sender.send(result);
    });
    if self
      .gpu
      .device
      .poll(wgpu::PollType::wait_indefinitely())
      .is_err()
      || !matches!(receiver.recv(), Ok(Ok(())))
    {
      return false;
    }
    let Ok(mapped) = slice.get_mapped_range() else {
      return false;
    };
    let (rows, _) = mapped.as_chunks::<{ ROW_PITCH as usize }>();
    for (source, destination) in rows.iter().zip(out.chunks_exact_mut(row)) {
      destination.copy_from_slice(&source[..row]);
    }
    drop(mapped);
    self.readback.unmap();
    true
  }

  /// Submits the scale and the copy into the readback buffer. False for a
  /// pixel format the camera is never asked for.
  fn draw(&mut self, buffer: &cv::PixelBuf, width: u32, height: u32) -> Result<bool, String> {
    let gpu = self.gpu;
    let format = buffer.pixel_format();
    let biplanar = format == cv::PixelFormat::_420V || format == cv::PixelFormat::_420F;
    if !biplanar && format != cv::PixelFormat::_32_BGRA {
      return Ok(false);
    }
    let (first, second) = if biplanar {
      (
        buffer_plane_texture(gpu, buffer, 0, wgpu::TextureFormat::R8Unorm, "camera luma")?,
        Some(buffer_plane_texture(
          gpu,
          buffer,
          1,
          wgpu::TextureFormat::Rg8Unorm,
          "camera chroma",
        )?),
      )
    } else {
      (
        buffer_plane_texture(
          gpu,
          buffer,
          0,
          wgpu::TextureFormat::Bgra8Unorm,
          "camera frame",
        )?,
        None,
      )
    };
    let first = first.create_view(&Default::default());
    let second = second.map(|texture| texture.create_view(&Default::default()));
    gpu.queue.write_buffer(
      &self.uniforms,
      0,
      bytemuck::bytes_of(&Uniforms {
        size: [width, height],
        full_range: u32::from(format == cv::PixelFormat::_420F),
        spare: 0,
      }),
    );
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide camera thumbnail"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: self.uniforms.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&first),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::TextureView(second.as_ref().unwrap_or(&first)),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    });
    let target = self.target(width, height);
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide camera thumbnail"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide camera thumbnail"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target.create_view(&Default::default()),
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(if biplanar { &self.biplanar } else { &self.bgra });
      pass.set_bind_group(0, &bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
      target.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: &self.readback,
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(ROW_PITCH),
          rows_per_image: Some(height),
        },
      },
      target.size(),
    );
    gpu.queue.submit([encoder.finish()]);
    Ok(true)
  }

  fn target(&mut self, width: u32, height: u32) -> wgpu::Texture {
    let gpu = self.gpu;
    self
      .target
      .take_if(|target| (target.width(), target.height()) != (width, height));
    self
      .target
      .get_or_insert_with(|| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
          label: Some("Screenwide camera thumbnail"),
          size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: FORMAT,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
          view_formats: &[],
        })
      })
      .clone()
  }
}
