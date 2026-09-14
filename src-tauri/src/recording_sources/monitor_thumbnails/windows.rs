// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use image::RgbaImage;
use windows::Win32::Graphics::Gdi::{
  CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetWindowDC, ReleaseDC,
  SelectObject, SetBrushOrgEx, SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
  DIB_RGB_COLORS, HALFTONE, HBITMAP, HDC, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

/// Capture a monitor directly into a thumbnail-sized DIB.
///
/// `xcap::Monitor::capture_image` first creates a full-resolution WGC frame.
/// That is unnecessarily expensive for the monitor picker and can leave a
/// large graphics allocation resident after the image has been downscaled.
/// GDI's screen DC lets Windows perform the scale while copying into the
/// small destination bitmap instead.
pub(super) fn capture(monitor: &xcap::Monitor) -> Result<RgbaImage, String> {
  let width = monitor.width().map_err(|error| error.to_string())?;
  let height = monitor.height().map_err(|error| error.to_string())?;
  let (output_width, output_height) = super::thumbnail_dimensions(width, height)
    .ok_or_else(|| "The monitor has invalid dimensions".to_owned())?;
  let x = monitor.x().map_err(|error| error.to_string())?;
  let y = monitor.y().map_err(|error| error.to_string())?;

  unsafe {
    let desktop = GetDesktopWindow();
    let source = GetWindowDC(Some(desktop));
    if source.is_invalid() {
      return Err("Windows could not acquire the desktop device context".to_owned());
    }
    let source = ScreenDc {
      hwnd: desktop,
      hdc: source,
    };

    let destination = CreateCompatibleDC(Some(source.hdc));
    if destination.is_invalid() {
      return Err("Windows could not create the thumbnail device context".to_owned());
    }
    let destination = MemoryDc(destination);

    let bitmap_info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: output_width as i32,
        // A negative height requests a top down DIB, matching RgbaImage's
        // row order and avoiding a second full image-sized buffer.
        biHeight: -(output_height as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    let bitmap = CreateDIBSection(
      Some(destination.0),
      &bitmap_info,
      DIB_RGB_COLORS,
      &mut bits,
      None,
      0,
    )
    .map_err(|error| format!("Windows could not create the thumbnail bitmap: {error}"))?;
    let bitmap = Bitmap { handle: bitmap };
    if bits.is_null() {
      return Err("Windows returned no thumbnail bitmap storage".to_owned());
    }

    let len = (output_width as usize)
      .checked_mul(output_height as usize)
      .and_then(|pixels| pixels.checked_mul(4))
      .ok_or_else(|| "The thumbnail dimensions overflowed".to_owned())?;

    let previous = SelectObject(destination.0, bitmap.handle.into());
    if previous.is_invalid() {
      return Err("Windows could not select the thumbnail bitmap".to_owned());
    }

    // HALFTONE gives monitor previews a stable result when the source is a
    // high-DPI display. StretchBlt writes only to the small DIB above.
    if SetStretchBltMode(destination.0, HALFTONE) == 0
      || !SetBrushOrgEx(destination.0, 0, 0, None).as_bool()
      || !StretchBlt(
        destination.0,
        0,
        0,
        output_width as i32,
        output_height as i32,
        Some(source.hdc),
        x,
        y,
        width as i32,
        height as i32,
        SRCCOPY,
      )
      .as_bool()
    {
      SelectObject(destination.0, previous);
      return Err("Windows could not scale the monitor into a thumbnail".to_owned());
    }

    // Ensure the GDI batch has completed before copying pixels out of the
    // DIB section. This also keeps the bitmap lifetime explicit on all paths.
    if !GdiFlush().as_bool() {
      SelectObject(destination.0, previous);
      return Err("Windows could not flush the thumbnail drawing".to_owned());
    }

    let mut pixels = std::slice::from_raw_parts(bits.cast::<u8>(), len).to_vec();
    SelectObject(destination.0, previous);

    // GDI stores 32 bit BI_RGB pixels as BGRA. The alpha byte is undefined
    // for a screen DC, so normalize it while converting to image::RgbaImage.
    for pixel in pixels.chunks_exact_mut(4) {
      pixel.swap(0, 2);
      pixel[3] = 255;
    }
    RgbaImage::from_raw(output_width, output_height, pixels)
      .ok_or_else(|| "Windows returned an invalid thumbnail buffer".to_owned())
  }
}

struct ScreenDc {
  hwnd: windows::Win32::Foundation::HWND,
  hdc: HDC,
}

impl Drop for ScreenDc {
  fn drop(&mut self) {
    unsafe {
      let _ = ReleaseDC(Some(self.hwnd), self.hdc);
    }
  }
}

struct MemoryDc(HDC);

impl Drop for MemoryDc {
  fn drop(&mut self) {
    unsafe {
      let _ = DeleteDC(self.0);
    }
  }
}

struct Bitmap {
  handle: HBITMAP,
}

impl Drop for Bitmap {
  fn drop(&mut self) {
    unsafe {
      let _ = DeleteObject(self.handle.into());
    }
  }
}
