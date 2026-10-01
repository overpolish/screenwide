// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use windows::Win32::Graphics::Gdi::{DeleteObject, GetObjectW, BITMAP, HGDIOBJ};

#[test]
fn native_bitmaps_are_dpi_sized_and_premultiplied() {
  for size in [16, 20, 24, 32, 48] {
    let bitmap = make_bitmap(super::super::icons::SETTINGS, size).expect("DIB creation");
    let mut info = BITMAP::default();
    let got = unsafe {
      GetObjectW(
        HGDIOBJ(bitmap.0),
        std::mem::size_of::<BITMAP>() as i32,
        Some((&mut info as *mut BITMAP).cast()),
      )
    };
    assert_eq!(got, std::mem::size_of::<BITMAP>() as i32);
    assert_eq!(
      (info.bmWidth, info.bmHeight, info.bmBitsPixel),
      (size as i32, size as i32, 32)
    );
    let pixels =
      unsafe { std::slice::from_raw_parts(info.bmBits.cast::<u8>(), (size * size * 4) as usize) };
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    for pixel in pixels.chunks_exact(4) {
      assert!(pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]);
    }
    unsafe {
      assert!(DeleteObject(bitmap.into()).as_bool());
    }
    let mut deleted = BITMAP::default();
    assert_eq!(
      unsafe {
        GetObjectW(
          HGDIOBJ(bitmap.0),
          std::mem::size_of::<BITMAP>() as i32,
          Some((&mut deleted as *mut BITMAP).cast()),
        )
      },
      0
    );
  }
}
