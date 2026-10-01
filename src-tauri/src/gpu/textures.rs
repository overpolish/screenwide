// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving pixels between the CPU and textures on the shared device.

use std::sync::mpsc;

use super::Gpu;

impl Gpu {
  /// A sampled texture holding `pixels`: `size` is width, height and layers,
  /// and `pixels` the layers one after another, rows tightly packed.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(crate) fn texture_with_pixels(
    &self,
    label: &str,
    size: (u32, u32, u32),
    format: wgpu::TextureFormat,
    pixels: &[u8],
  ) -> wgpu::Texture {
    let extent = wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: size.2,
    };
    let texture = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: extent,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      // Copyable so a redaction pass can start from a screenshot's pixels.
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let bytes_per_pixel = format.block_copy_size(None).unwrap_or(4);
    self.queue.write_texture(
      texture.as_image_copy(),
      pixels,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(size.0 * bytes_per_pixel),
        rows_per_image: Some(size.1),
      },
      extent,
    );
    texture
  }

  /// The first layer of a four-byte-per-pixel `texture`, rows tightly packed,
  /// as stored. Waits for everything submitted so far.
  pub(crate) fn read_texture(&self, texture: &wgpu::Texture) -> Result<Vec<u8>, String> {
    let (width, height) = (texture.width(), texture.height());
    let row_bytes = width * 4;
    let padded_row_bytes = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide readback"),
      size: u64::from(padded_row_bytes) * u64::from(height),
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation: false,
    });
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide readback"),
      });
    encoder.copy_texture_to_buffer(
      texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: &buffer,
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(padded_row_bytes),
          rows_per_image: Some(height),
        },
      },
      wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
    );
    self.queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
      let _ = sender.send(result);
    });
    self
      .device
      .poll(wgpu::PollType::wait_indefinitely())
      .map_err(|error| error.to_string())?;
    receiver
      .recv()
      .map_err(|error| error.to_string())?
      .map_err(|error| error.to_string())?;
    let mapped = slice
      .get_mapped_range()
      .map_err(|error| error.to_string())?;
    let mut pixels = Vec::with_capacity(row_bytes as usize * height as usize);
    for row in mapped.chunks_exact(padded_row_bytes as usize) {
      pixels.extend_from_slice(&row[..row_bytes as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
  }
}
