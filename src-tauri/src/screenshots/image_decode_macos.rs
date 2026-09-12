// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! HEIC, read through the decoder the system already has.
//!
//! The `image` crate this build uses has no HEIC support, and the pictures the
//! system offers as desktop backgrounds are all HEIC, so the decode goes
//! through ImageIO and comes back as the RGBA8 the rest of the pipeline works
//! in. The scaling happens inside the decode rather than after it: a desktop
//! picture is tens of megapixels and a swatch is a few dozen pixels across.

use std::ffi::{c_char, CString};

unsafe extern "C" {
  fn screenwide_decode_image_rgba(
    path: *const c_char,
    max_pixel_size: u32,
    out_width: *mut u32,
    out_height: *mut u32,
  ) -> *mut u8;
  fn screenwide_free_decoded_image(pixels: *mut u8);
}

/// The picture at `path`, with its longer edge held to `max_pixel_size`.
///
/// `None` when the file cannot be read or decoded, which every caller answers
/// with the flat colour rather than with an error.
pub(crate) fn decode_rgba(path: &str, max_pixel_size: u32) -> Option<image::RgbaImage> {
  let file = CString::new(path).ok()?;
  let mut width: u32 = 0;
  let mut height: u32 = 0;
  // SAFETY: the path outlives the call, and the two out parameters are live
  // locals the native side only writes once it has a picture to describe.
  let pixels = unsafe {
    screenwide_decode_image_rgba(
      file.as_ptr(),
      max_pixel_size.max(1),
      &raw mut width,
      &raw mut height,
    )
  };
  if pixels.is_null() {
    return None;
  }
  let length = (width as usize) * (height as usize) * 4;
  let copied = if length == 0 {
    None
  } else {
    // SAFETY: the native side returns a tightly packed RGBA8 buffer of
    // exactly `width * height * 4` bytes, which it keeps alive until it is
    // freed on the next line.
    Some(unsafe { std::slice::from_raw_parts(pixels, length) }.to_vec())
  };
  // SAFETY: the pointer came from the matching allocator on the native side
  // and is not used again.
  unsafe { screenwide_free_decoded_image(pixels) };
  image::RgbaImage::from_raw(width, height, copied?)
}
