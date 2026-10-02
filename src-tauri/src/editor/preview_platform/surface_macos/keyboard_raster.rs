// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Core Graphics rasterisation of the keyboard-shortcut strip: macOS key
//! symbols set in Inter, matching the React keycap. The drawing is
//! `screenwide_keyboard_raster` in `gpu_compositor_macos_keyboard_artwork.m`.

use super::super::keyboard_artwork::KeyboardRaster;

const MAX_KEYS: usize = 8;

/// The twin of `ScreenwideKeyboardRaster`.
#[repr(C)]
struct NativeKeyboardRaster {
  pixels: *mut u8,
  width: u32,
  height: u32,
  key_count: u32,
  key_x: [u32; MAX_KEYS],
  key_width: [u32; MAX_KEYS],
}

unsafe extern "C" {
  fn screenwide_keyboard_raster(
    codes: *const u16,
    count: u32,
    light: u32,
    backing_scale: f64,
    out: *mut NativeKeyboardRaster,
  ) -> i32;
  fn screenwide_keyboard_raster_free(raster: *mut NativeKeyboardRaster);
}

/// The strip for the keys `codes`, in the light or dark appearance, at
/// `backing_scale` pixels per design point, as premultiplied BGRA.
pub(crate) fn rasterize_keyboard(
  codes: &[u16],
  light: bool,
  backing_scale: f64,
) -> Result<KeyboardRaster, String> {
  if codes.is_empty() || codes.len() > MAX_KEYS {
    return Err("The keyboard shortcut has no keys to draw".to_owned());
  }
  let mut native = NativeKeyboardRaster {
    pixels: std::ptr::null_mut(),
    width: 0,
    height: 0,
    key_count: 0,
    key_x: [0; MAX_KEYS],
    key_width: [0; MAX_KEYS],
  };
  let drawn = unsafe {
    screenwide_keyboard_raster(
      codes.as_ptr(),
      codes.len() as u32,
      u32::from(light),
      backing_scale,
      &mut native,
    )
  };
  if drawn == 0 || native.pixels.is_null() {
    return Err("Core Graphics could not draw the keyboard shortcut".to_owned());
  }
  let length = native.width as usize * native.height as usize * 4;
  let mut pixels = unsafe { std::slice::from_raw_parts(native.pixels, length) }.to_vec();
  unsafe { screenwide_keyboard_raster_free(&mut native) };
  // Core Graphics drew RGBA; the strip is uploaded as BGRA.
  for pixel in pixels.as_chunks_mut::<4>().0 {
    pixel.swap(0, 2);
  }
  Ok(KeyboardRaster {
    pixels,
    size: (native.width, native.height),
    keys: (0..native.key_count as usize)
      .map(|index| (native.key_x[index], native.key_width[index]))
      .collect(),
  })
}
