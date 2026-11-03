// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_int;

use super::CapturedImage;

/// How finely a screenshot saved as a WebP keeps its detail, on libwebp's
/// 0 to 100 scale: the JPEG's quality, so text keeps clean edges.
const WEBP_QUALITY: f32 = 90.0;

/// The longest side libwebp can write, in pixels.
const WEBP_MAX_SIDE: u32 = 16_383;

/// The image as a lossy WebP. Unlike a JPEG it keeps what the image leaves
/// see-through, such as rounded corners; an opaque image is written without
/// an alpha channel.
pub fn encode_webp(image: &CapturedImage) -> Result<Vec<u8>, String> {
  if image.width > WEBP_MAX_SIDE || image.height > WEBP_MAX_SIDE {
    return Err(format!(
      "WebP cannot hold a picture over {WEBP_MAX_SIDE} pixels on a side; this one is {} by {}",
      image.width, image.height
    ));
  }
  let expected = image.width as usize * image.height as usize * 4;
  if image.width == 0 || image.height == 0 || image.rgba.len() != expected {
    return Err("The screenshot pixels are not valid".to_owned());
  }
  let mut output = std::ptr::null_mut();
  // Safety: the pixels are `width` by `height` RGBA rows of `width * 4`
  // bytes, checked above, and both sides fit a C int. libwebp allocates the
  // output, which is copied and then freed with its own allocator.
  unsafe {
    let size = libwebp_sys::WebPEncodeRGBA(
      image.rgba.as_ptr(),
      image.width as c_int,
      image.height as c_int,
      (image.width * 4) as c_int,
      WEBP_QUALITY,
      &mut output,
    );
    if size == 0 || output.is_null() {
      return Err("The screenshot could not be encoded as WebP".to_owned());
    }
    let bytes = std::slice::from_raw_parts(output, size).to_vec();
    libwebp_sys::WebPFree(output.cast());
    Ok(bytes)
  }
}
