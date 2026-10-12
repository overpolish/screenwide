// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as a drawing applies them: what is behind the
//! person, the grade, the picture, and the matte to read beside the camera.
//! The app loads them from the project whenever they change; the preview,
//! the export, the thumbnails and the stills read them alike.

use std::path::PathBuf;
use std::sync::Arc;

use super::grade::Grade;
use super::CameraBackground;

/// The matte file, and when each of its frames' camera frames shows.
#[derive(Clone, Debug)]
pub struct MatteFile {
  pub path: PathBuf,
  pub timings: Arc<screenwide_transcriber::MatteTimings>,
}

impl MatteFile {
  /// How long the matte file runs, in its own milliseconds.
  pub(crate) fn duration_ms(&self) -> u64 {
    (self.timings.frames_ms.len() as u64 * 1_000)
      .div_ceil(u64::from(self.timings.frames_per_second))
  }

  /// Where in the matte file the matte of the camera frame showing at
  /// camera time `ms` is: the start of that frame's slot, in its own
  /// milliseconds.
  pub(crate) fn matte_ms(&self, ms: u64) -> u64 {
    // A frame shows from its own time; the decoders hand over a frame
    // stamped up to 2 ms after the moment asked for, so the matte does too.
    let shown = self
      .timings
      .frames_ms
      .partition_point(|&frame_ms| frame_ms <= ms as f64 + 2.0)
      .saturating_sub(1);
    (shown as u64 * 1_000).div_ceil(u64::from(self.timings.frames_per_second))
  }
}

/// The effects one drawing applies to the camera.
pub struct LiveCameraEffects {
  pub background: CameraBackground,
  pub grade: Option<Grade>,
  pub picture: Option<Arc<::image::RgbaImage>>,
  pub matte: Option<MatteFile>,
  /// Sharpened, and enlarged along its edges where drawn larger.
  pub enhance: bool,
  /// Different for every load, so what is cached from one set of effects,
  /// such as the picture on the GPU, is not drawn with the next.
  pub generation: u64,
}
