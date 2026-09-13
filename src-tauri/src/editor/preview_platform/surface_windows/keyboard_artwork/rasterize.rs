// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl TextDevice {
  fn new(backing_scale: f64) -> Result<Self, String> {
    super::super::font::register_inter_font();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return Err("Windows could not create a keyboard artwork drawing context".to_owned());
    }
    let face: Vec<u16> = "Inter\0".encode_utf16().collect();
    let font = unsafe {
      CreateFontW(
        -(((FONT_SIZE * backing_scale).round() as i32).max(1)),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
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
      return Err("Windows could not create the keyboard artwork font".to_owned());
    }
    let old_font = unsafe { SelectObject(dc, font.into()) };
    unsafe { SetTextCharacterExtra(dc, 0) };
    Ok(Self { dc, font, old_font })
  }

  fn measure(&self, text: &[u16]) -> Result<(i32, i32), String> {
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, text, &mut extent) };
    if !measured.as_bool() || extent.cx <= 0 || extent.cy <= 0 {
      return Err("Windows could not measure a keyboard shortcut label".to_owned());
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

fn rounded_coverage(x: f64, y: f64, left: f64, width: f64, height: f64, radius: f64) -> f64 {
  let half_width = width * 0.5;
  let half_height = height * 0.5;
  let radius = radius.min(half_width).min(half_height).max(0.0);
  let local_x = (x - (left + half_width)).abs() - (half_width - radius);
  let local_y = (y - half_height).abs() - (half_height - radius);
  let outside = local_x.max(0.0).hypot(local_y.max(0.0));
  let distance = outside + local_x.max(local_y).min(0.0) - radius;
  (0.5 - distance).clamp(0.0, 1.0)
}

/// Rasterises the shortcut strip at device pixels per design point.
pub(in crate::editor::preview_platform::surface) fn rasterize_keyboard(
  labels: &[String],
  light: bool,
  backing_scale: f64,
) -> Result<KeyboardRaster, String> {
  if labels.is_empty() {
    return Err("The keyboard shortcut has no keys to draw".to_owned());
  }
  let device = TextDevice::new(backing_scale)?;
  let wide: Vec<Vec<u16>> = labels
    .iter()
    .map(|label| label.encode_utf16().collect())
    .collect();
  let mut measured = Vec::with_capacity(labels.len());
  for text in &wide {
    measured.push(device.measure(text)?);
  }
  let text_widths: Vec<f64> = measured
    .iter()
    .zip(labels)
    .map(|((cx, _), label)| {
      if label == "⇧" {
        12.0
      } else {
        f64::from(*cx) / backing_scale
      }
    })
    .collect();
  let key_widths: Vec<f64> = text_widths
    .iter()
    .map(|width| (width.ceil() + DESIGN_INSET * 2.0).max(DESIGN_MINIMUM_WIDTH))
    .collect();
  let design_width =
    key_widths.iter().sum::<f64>() + DESIGN_GAP * (key_widths.len().saturating_sub(1)) as f64;
  let width = ((design_width * backing_scale).ceil() as u32).max(1);
  let height = ((DESIGN_HEIGHT * backing_scale).ceil() as u32).max(1);

  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width as i32,
      biHeight: -(height as i32),
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
  let length = (width as usize) * (height as usize) * 4;
  let raster = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
  for pixel in raster.chunks_exact_mut(4) {
    pixel.fill(0);
  }
  let mut key_x = 0.0_f64;
  let mut drawn = true;
  unsafe {
    SetBkMode(device.dc, TRANSPARENT);
    SetTextColor(device.dc, COLORREF(0x00FF_FFFF));
  }
  let mut keys = Vec::with_capacity(labels.len());
  for (index, text) in wide.iter().enumerate() {
    let key_width = key_widths[index];
    let text_x = key_x + (key_width - text_widths[index]) * 0.5;
    let y = (height as i32 - measured[index].1) / 2;
    drawn &= if labels[index] == "⇧" {
      true // Modifier coverage is drawn below, independently of GDI text.
    } else {
      unsafe { TextOutW(device.dc, (text_x * backing_scale).round() as i32, y, text) }.as_bool()
    };
    keys.push((
      (key_x * backing_scale).round() as u32,
      (key_width * backing_scale).round() as u32,
    ));
    key_x += key_width + DESIGN_GAP;
  }
  let mut coverage: Vec<u8> = raster.chunks_exact(4).map(|pixel| pixel[2]).collect();
  icons::draw_modifiers(&mut coverage, width, height, &keys, labels, backing_scale);
  unsafe {
    SelectObject(device.dc, old_bitmap);
    let _ = DeleteObject(bitmap.into());
  }
  if !drawn {
    return Err("Windows could not draw a keyboard shortcut label".to_owned());
  }

  // Resolve bg-fill over the Storybook window backing (#FFFFFF / #252525).
  let (backing, foreground) = if light { (255.0, 0.0) } else { (37.0, 255.0) };
  let background = backing * (1.0 - FILL_ALPHA) + foreground * FILL_ALPHA;
  let radius = DESIGN_RADIUS * backing_scale;
  let mut pixels = vec![0_u8; length];
  for (cap_x, cap_pixels) in &keys {
    let left = f64::from(*cap_x);
    let cap_width = f64::from(*cap_pixels);
    let first = cap_x.saturating_sub(2);
    let last = (cap_x + cap_pixels + 2).min(width);
    for row in 0..height {
      let y = f64::from(row) + 0.5;
      for column in first..last {
        let cap = rounded_coverage(
          f64::from(column) + 0.5,
          y,
          left,
          cap_width,
          f64::from(height),
          radius,
        );
        if cap <= 0.0 {
          continue;
        }
        let offset = ((row as usize) * (width as usize) + column as usize) * 4;
        let value = (background * cap).round().clamp(0.0, 255.0) as u8;
        pixels[offset] = value;
        pixels[offset + 1] = value;
        pixels[offset + 2] = value;
        pixels[offset + 3] = (cap * 255.0).round().clamp(0.0, 255.0) as u8;
      }
    }
  }
  for (pixel, text) in pixels.chunks_exact_mut(4).zip(coverage) {
    if text == 0 {
      continue;
    }
    let text = f64::from(text) / 255.0 * TEXT_ALPHA;
    let keep = 1.0 - text;
    let alpha = text + (f64::from(pixel[3]) / 255.0) * keep;
    for channel in pixel[..3].iter_mut() {
      let value = foreground * text + f64::from(*channel) * keep;
      *channel = value.round().clamp(0.0, 255.0) as u8;
    }
    pixel[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
  }
  Ok(KeyboardRaster {
    pixels,
    size: (width, height),
    keys,
  })
}
