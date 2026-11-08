// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! BGRA textures in, an MP4 out: what a recording or an export writes.

use std::path::Path;

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};

use crate::{Converter, Crop, EncodedFrame, Encoder, Mp4Writer, Picture, Settings, SLOTS};

pub struct FileEncoder {
  converter: Converter,
  encoded: Vec<EncodedFrame>,
  encoder: Encoder,
  next_slot: usize,
  writer: Mp4Writer,
}

impl FileEncoder {
  /// Creates the file at `path`. `crop` is the part of each frame kept, the
  /// whole frame when `None`; `fragment_100ns` the shortest fragment.
  pub fn create(
    device: &ID3D11Device,
    settings: Settings,
    crop: Option<Crop>,
    path: &Path,
    fragment_100ns: Option<u64>,
  ) -> Result<Self, String> {
    let encoder = Encoder::new(device, settings)?;
    let converter = Converter::new(
      device,
      settings.width,
      settings.height,
      settings.fps,
      crop,
      encoder.input_bind_flags(),
    )?;
    let writer = Mp4Writer::create(path, settings, fragment_100ns)?;
    Ok(Self {
      converter,
      encoded: Vec::new(),
      encoder,
      next_slot: 0,
      writer,
    })
  }

  /// Encodes `texture`, which may be reused as soon as this returns: it is
  /// copied into a picture of the encoder's own first.
  pub fn write(
    &mut self,
    texture: &ID3D11Texture2D,
    pts_100ns: i64,
    duration_100ns: i64,
  ) -> Result<(), String> {
    let slot = self.free_slot()?;
    self.converter.convert(texture, slot)?;
    self.encoder.encode(
      self.converter.texture(slot),
      slot,
      pts_100ns,
      duration_100ns.max(1),
      Picture::Any,
      &mut self.encoded,
    )?;
    self.write_encoded()
  }

  /// Writes every frame still in the encoder and closes the file.
  pub fn finish(&mut self) -> Result<(), String> {
    self.encoder.finish(&mut self.encoded)?;
    self.write_encoded()?;
    self.writer.finish()
  }

  /// A picture the encoder is not reading, waiting for it to hand frames
  /// back when it holds all of them.
  fn free_slot(&mut self) -> Result<usize, String> {
    for _ in 0..2 {
      for _ in 0..SLOTS {
        let slot = self.next_slot;
        self.next_slot = (slot + 1) % SLOTS;
        if !self.encoder.holds(slot) {
          return Ok(slot);
        }
      }
      self.encoder.flush(&mut self.encoded)?;
      self.write_encoded()?;
    }
    Err("The encoder kept every picture it was given".to_owned())
  }

  fn write_encoded(&mut self) -> Result<(), String> {
    for frame in self.encoded.drain(..) {
      self.writer.write(&frame)?;
    }
    Ok(())
  }
}
