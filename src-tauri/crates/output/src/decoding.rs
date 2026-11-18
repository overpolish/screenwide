// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot read back from its file.

use std::path::Path;

use image::{ColorType, DynamicImage, ImageDecoder, ImageReader};

use super::encoding::reserved;
use super::CapturedImage;

/// The picture in the file at `path`, as RGBA. A screenshot is kept as RGBA,
/// which is decoded straight into a buffer asked for first, so one there is
/// not the memory for is an error rather than the end of the app; a picture
/// kept any other way is converted after decoding.
pub fn read_rgba(path: &Path) -> Result<CapturedImage, String> {
  let decoder = ImageReader::open(path)
    .and_then(ImageReader::with_guessed_format)
    .map_err(|error| error.to_string())?
    .into_decoder()
    .map_err(|error| error.to_string())?;
  let (width, height) = decoder.dimensions();
  if decoder.color_type() != ColorType::Rgba8 {
    let picture = DynamicImage::from_decoder(decoder)
      .map_err(|error| error.to_string())?
      .into_rgba8();
    return Ok(CapturedImage {
      width,
      height,
      rgba: picture.into_raw(),
    });
  }
  let length = usize::try_from(decoder.total_bytes())
    .map_err(|_| "The picture is too large to hold".to_owned())?;
  let mut rgba = reserved(length)?;
  rgba.resize(length, 0);
  decoder
    .read_image(&mut rgba)
    .map_err(|error| error.to_string())?;
  Ok(CapturedImage {
    width,
    height,
    rgba,
  })
}
