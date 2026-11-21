// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A small picture drawn four times as large by the upscaling model.

use std::path::PathBuf;

use screenwide_transcriber::{Job, Reply};

use super::Transcriber;

impl Transcriber {
  /// Draws the picture in the file `image`, `width` by `height` pixels of
  /// red, green, blue and alpha bytes, four times as large with the model at
  /// `model`, into `output` in the same form, telling `progress` how far it
  /// has got.
  pub fn upscale(
    &mut self,
    model: PathBuf,
    (image, width, height): (PathBuf, u32, u32),
    output: PathBuf,
    progress: &mut dyn FnMut(f32),
  ) -> Result<(), String> {
    let job = Job::Upscale {
      model,
      image,
      width,
      height,
      output,
    };
    match self.ask(job, progress)? {
      Reply::Upscaled { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
