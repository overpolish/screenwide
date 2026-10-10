// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each object's masks, found out of order, kept in a file of fixed-size
//! slots until the last is in, then written in turn as a grey video.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::super::encode::Encoder;
use super::cutie::{MASK_HEIGHT, MASK_WIDTH};

const MASK_BYTES: usize = MASK_WIDTH * MASK_HEIGHT;

/// Each object's masks, found out of order, waiting in a file of fixed-size
/// slots until the last is in; slot `n` holds camera frame `first + n`.
pub(super) struct Slots {
  /// Each object's slots, and where its masks go.
  files: Vec<(File, PathBuf, PathBuf)>,
  first: usize,
}

impl Slots {
  pub(super) fn create(outputs: &[&Path], first: usize) -> Result<Self, String> {
    let files = outputs
      .iter()
      .map(|output| {
        let path = output.with_extension("slots");
        File::options()
          .read(true)
          .write(true)
          .create(true)
          .truncate(true)
          .open(&path)
          .map(|file| (file, path, output.to_path_buf()))
          .map_err(|error| format!("Could not write the object's masks: {error}"))
      })
      .collect::<Result<Vec<_>, String>>()?;
    Ok(Self { files, first })
  }

  pub(super) fn write(&mut self, object: usize, frame: usize, mask: &[f32]) -> Result<(), String> {
    let bytes = mask
      .iter()
      .map(|&value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
      .collect::<Vec<_>>();
    let file = &mut self.files[object].0;
    file
      .seek(SeekFrom::Start(((frame - self.first) * MASK_BYTES) as u64))
      .and_then(|_| file.write_all(&bytes))
      .map_err(|error| format!("Could not write the object's masks: {error}"))
  }

  /// The masks written in turn as grey videos, and the slots taken away.
  /// Frames never written, past where following stopped, are empty.
  pub(super) fn finish(self, ffmpeg: &Path, frames: usize) -> Result<(), String> {
    let mut written = Ok(());
    for (mut file, path, output) in self.files {
      if written.is_ok() {
        written = file
          .set_len((frames * MASK_BYTES) as u64)
          .map_err(|error| format!("Could not write the object's masks: {error}"));
      }
      if written.is_ok() {
        written = encode(ffmpeg, &mut file, frames, &output);
      }
      drop(file);
      let _ = std::fs::remove_file(path);
    }
    written
  }
}

fn encode(ffmpeg: &Path, slots: &mut File, frames: usize, output: &Path) -> Result<(), String> {
  let size = (MASK_WIDTH as u32, MASK_HEIGHT as u32);
  let mut encoder = Encoder::start(ffmpeg, size, 30, output)?;
  let mut mask = vec![0_u8; MASK_BYTES];
  slots
    .seek(SeekFrom::Start(0))
    .map_err(|error| error.to_string())?;
  for _ in 0..frames {
    slots
      .read_exact(&mut mask)
      .map_err(|error| format!("Could not read the object's masks: {error}"))?;
    encoder.write(&mask)?;
  }
  encoder.finish()
}
