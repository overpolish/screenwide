// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation of the counters' numbers, the twin of the Core Text
//! pass in `gpu_compositor_macos_annotation_text.m`, and the cell every piece
//! of annotation type is drawn into.

use super::*;
use crate::editor::preview_platform::surface::type_device::TypeDevice;

/// One number set for its cell: the text, the size it measures at, and the
/// cell it is centred in. A number too wide for its disc is narrowed rather
/// than allowed to touch the edge.
struct Row {
  text: Vec<u16>,
  measured: (i32, i32),
  cell: (u32, u32),
  size: f64,
}

fn row(value: &str, radius: f32) -> Result<Row, String> {
  let diameter = f64::from(radius) * 2.0 * SUPERSAMPLE;
  let mut size = diameter * CAP_SHARE / CAP_HEIGHT;
  let text: Vec<u16> = value.encode_utf16().collect();
  let mut measured = TypeDevice::new(size)?.measure(&text)?;
  let limit = diameter * WIDTH_SHARE;
  if f64::from(measured.0) > limit && measured.0 > 0 {
    size *= limit / f64::from(measured.0);
    measured = TypeDevice::new(size)?.measure(&text)?;
  }
  Ok(Row {
    // One transparent pixel of margin keeps a four-tap sample on one number.
    cell: (
      (measured.0 as u32).saturating_add(2).max(1),
      (measured.1 as u32).saturating_add(2).max(1),
    ),
    measured,
    size,
    text,
  })
}

/// The cell `value` needs at `radius`, in atlas pixels.
pub(super) fn measure(value: &str, radius: f32) -> Result<(u32, u32), String> {
  Ok(row(value, radius)?.cell)
}

/// `value` drawn at `radius` into a cell of `cell` pixels, as BGRA rows, top
/// row first.
pub(super) fn draw(value: &str, radius: f32, cell: (u32, u32)) -> Result<Vec<u8>, String> {
  let row = row(value, radius)?;
  // GDI selects a font into a context rather than into a draw, so the number
  // gets a context of its own at its own size.
  let device = TypeDevice::new(row.size)?;
  raster_cell(&device, cell, |dc, _| {
    let x = (cell.0 as i32 - row.measured.0) / 2;
    let y = (cell.1 as i32 - row.measured.1) / 2;
    unsafe { TextOutW(dc, x, y, &row.text) }.as_bool()
  })
}

/// One cell painted by `paint` in white over nothing, as BGRA rows, top row
/// first. The type is tinted by the shader, so its coverage is carried in the
/// alpha channel over white. `paint` is handed the context and the cell's
/// rows, which it may mark directly - a caret or a selection is coverage the
/// text engine does not draw - and answers whether it drew.
pub(super) fn raster_cell(
  device: &TypeDevice,
  cell: (u32, u32),
  paint: impl FnOnce(HDC, &mut [u8]) -> bool,
) -> Result<Vec<u8>, String> {
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: cell.0 as i32,
      // Negative: the rows run top-down from the first pixel, which is the
      // space the shader samples the rectangles in.
      biHeight: -(cell.1 as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut bits = std::ptr::null_mut();
  let bitmap = unsafe {
    CreateDIBSection(
      Some(device.dc),
      &raw const info,
      DIB_RGB_COLORS,
      &mut bits,
      None,
      0,
    )
  }
  .map_err(|error| error.to_string())?;
  let old_bitmap = unsafe { SelectObject(device.dc, bitmap.into()) };
  let length = cell.0 as usize * cell.1 as usize * 4;
  let raster = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
  raster.fill(0);
  unsafe {
    SetBkMode(device.dc, TRANSPARENT);
    SetTextColor(device.dc, COLORREF(0x00FF_FFFF));
  }
  let drawn = paint(device.dc, raster);
  // GDI writes no alpha itself: the white text's red channel is its coverage.
  let pixels: Vec<u8> = raster
    .chunks_exact(4)
    .flat_map(|pixel| [255, 255, 255, pixel[2]])
    .collect();
  unsafe {
    SelectObject(device.dc, old_bitmap);
    let _ = DeleteObject(bitmap.into());
  }
  if !drawn {
    return Err("Windows could not draw annotation type".to_owned());
  }
  Ok(pixels)
}
