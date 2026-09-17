// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation of the counters' numbers, the twin of the Core Text
//! pass in `gpu_compositor_macos_annotation_text.m`.

use super::*;

struct TextDevice {
  dc: HDC,
  font: HFONT,
  old_font: HGDIOBJ,
}

impl TextDevice {
  /// Inter semibold at `size` atlas pixels. The face is registered with GDI
  /// by the preview surface's own font module, which the keyboard artwork
  /// shares.
  fn new(size: f64) -> Result<Self, String> {
    super::font::register_inter_font();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return Err("Windows could not create a counter drawing context".to_owned());
    }
    let face: Vec<u16> = "Inter\0".encode_utf16().collect();
    let font = unsafe {
      CreateFontW(
        -((size.round() as i32).max(1)),
        0,
        0,
        0,
        FW_SEMIBOLD.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        ANTIALIASED_QUALITY,
        (VARIABLE_PITCH.0 | FF_SWISS.0) as u32,
        PCWSTR(face.as_ptr()),
      )
    };
    if font.is_invalid() {
      let _ = unsafe { DeleteDC(dc) };
      return Err("Windows could not create the counter font".to_owned());
    }
    let old_font = unsafe { SelectObject(dc, font.into()) };
    Ok(Self { dc, font, old_font })
  }

  fn measure(&self, text: &[u16]) -> Result<(i32, i32), String> {
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, text, &mut extent) };
    if !measured.as_bool() || extent.cx <= 0 || extent.cy <= 0 {
      return Err("Windows could not measure a counter's number".to_owned());
    }
    Ok((extent.cx, extent.cy))
  }
}

impl Drop for TextDevice {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.dc, self.old_font);
      let _ = DeleteObject(self.font.into());
      let _ = DeleteDC(self.dc);
    }
  }
}

struct CounterRaster {
  pixels: Vec<u8>,
  rects: Vec<CounterTextRect>,
  size: (u32, u32),
}

/// One number's own row: the text, the size it measures at, and the cell it
/// is centred in. A number too wide for its disc is narrowed rather than
/// allowed to touch the edge.
struct Row {
  mark: usize,
  text: Vec<u16>,
  measured: (i32, i32),
  cell: (u32, u32),
  size: f64,
}

fn row(mark: usize, value: u32, radius: f64) -> Result<Row, String> {
  let diameter = radius * 2.0 * SUPERSAMPLE;
  let mut size = diameter * CAP_SHARE / CAP_HEIGHT;
  let text: Vec<u16> = format!("{value}").encode_utf16().collect();
  let mut measured = TextDevice::new(size)?.measure(&text)?;
  let limit = diameter * WIDTH_SHARE;
  if f64::from(measured.0) > limit && measured.0 > 0 {
    size *= limit / f64::from(measured.0);
    measured = TextDevice::new(size)?.measure(&text)?;
  }
  Ok(Row {
    mark,
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

fn rasterize(wanted: &[(usize, u32, f64)], marks: usize) -> Result<CounterRaster, String> {
  let mut rows = Vec::with_capacity(wanted.len());
  for (mark, value, radius) in wanted {
    rows.push(row(*mark, *value, *radius)?);
  }
  let width = rows.iter().map(|row| row.cell.0).max().unwrap_or(1).max(1);
  let height = rows.iter().map(|row| row.cell.1).sum::<u32>().max(1);
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width as i32,
      // Negative: the rows run top-down from the first pixel, which is the
      // space the shader samples the rectangles in.
      biHeight: -(height as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut rects = vec![CounterTextRect::default(); marks];
  let mut pixels = Vec::new();
  let mut top = 0_u32;
  // One context per row: each number is set at its own size, and GDI selects
  // a font into a context rather than into a draw.
  for row in &rows {
    let device = TextDevice::new(row.size)?;
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
    let length = (width as usize) * (row.cell.1 as usize) * 4;
    let raster = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
    raster.fill(0);
    unsafe {
      SetBkMode(device.dc, TRANSPARENT);
      SetTextColor(device.dc, COLORREF(0x00FF_FFFF));
    }
    let x = (row.cell.0 as i32 - row.measured.0) / 2;
    let y = (row.cell.1 as i32 - row.measured.1) / 2;
    let drawn = unsafe { TextOutW(device.dc, x, y, &row.text) }.as_bool();
    // The number is tinted by the shader, so the coverage is carried in the
    // alpha channel and the colour left white: GDI writes no alpha itself.
    let mut written: Vec<u8> = raster
      .chunks_exact(4)
      .flat_map(|pixel| [255, 255, 255, pixel[2]])
      .collect();
    unsafe {
      SelectObject(device.dc, old_bitmap);
      let _ = DeleteObject(bitmap.into());
    }
    if !drawn {
      return Err("Windows could not draw a counter's number".to_owned());
    }
    pixels.append(&mut written);
    rects[row.mark] = CounterTextRect {
      x: 0.0,
      y: top as f32,
      width: row.cell.0 as f32,
      height: row.cell.1 as f32,
    };
    top += row.cell.1;
  }
  Ok(CounterRaster {
    pixels,
    rects,
    size: (width, height),
  })
}
