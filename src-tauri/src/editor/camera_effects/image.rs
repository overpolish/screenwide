// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The picture behind the person, kept in the project so the project carries
//! it when it moves. It is kept as a JPEG whatever format it was chosen in:
//! the FFmpeg bundled on macOS is built without a PNG decoder, and reads
//! JPEG through its MJPEG one.

use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// The longest side kept: no camera is drawn larger, enhanced or not.
const LONGEST_SIDE: u32 = 3_840;
/// Near enough lossless for a picture that is only ever a backdrop.
const QUALITY: u8 = 95;

pub(super) fn path(folder: &Path) -> PathBuf {
  folder.join("camera-background.jpg")
}

/// Keeps the picture at `chosen` in `folder`, in place of any before it.
pub(super) fn import(chosen: &Path, folder: &Path) -> Result<(), String> {
  let picture =
    ::image::open(chosen).map_err(|error| format!("Could not read that picture: {error}"))?;
  let picture = if picture.width().max(picture.height()) > LONGEST_SIDE {
    picture.resize(
      LONGEST_SIDE,
      LONGEST_SIDE,
      ::image::imageops::FilterType::Lanczos3,
    )
  } else {
    picture
  };
  let unwritable =
    |error: &dyn std::fmt::Display| format!("Could not keep the background picture: {error}");
  let destination = path(folder);
  let partial = destination.with_extension("partial.jpg");
  let file = std::fs::File::create(&partial).map_err(|error| unwritable(&error))?;
  ::image::codecs::jpeg::JpegEncoder::new_with_quality(BufWriter::new(file), QUALITY)
    .encode_image(&picture.to_rgb8())
    .map_err(|error| unwritable(&error))?;
  std::fs::rename(&partial, &destination).map_err(|error| unwritable(&error))
}
