// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Context {
  pub(super) fn new(font_size: f64, raster_scale: f64, mono: bool) -> Option<Self> {
    if font_size <= 0.0 || raster_scale <= 0.0 {
      return None;
    }
    register_fonts();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return None;
    }
    // The page's `--font-sans` on the Windows skin is Segoe UI Variable, but
    // GDI does not resolve the variable family by name and substitutes a
    // stranger; the static Segoe UI is the same design at these sizes.
    let face: Vec<u16> = if mono { "Roboto Mono\0" } else { "Segoe UI\0" }
      .encode_utf16()
      .collect();
    let family = if mono {
      FIXED_PITCH.0 | FF_MODERN.0
    } else {
      VARIABLE_PITCH.0 | FF_SWISS.0
    };
    let font = unsafe {
      CreateFontW(
        -((font_size * raster_scale).round() as i32).max(1),
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
        u32::from(family),
        PCWSTR(face.as_ptr()),
      )
    };
    if font.is_invalid() {
      let _ = unsafe { DeleteDC(dc) };
      return None;
    }
    let previous = unsafe { SelectObject(dc, font.into()) };
    Some(Self { dc, font, previous })
  }

  /// Glyph cell extent in raster pixels.
  pub(super) fn measure(&self, text: &str) -> Option<(i32, i32)> {
    let wide: Vec<u16> = text.encode_utf16().collect();
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, &wide, &mut extent) };
    (measured.as_bool() && extent.cx > 0 && extent.cy > 0).then_some((extent.cx, extent.cy))
  }

  /// Draws `runs` (text plus a raster-pixel origin) white on black and returns
  /// the box-downsampled coverage at `output` resolution. GDI antialiasing
  /// blends white glyphs into the black ground, so the red channel is plain
  /// coverage.
  pub(super) fn coverage(
    &self,
    output: (i32, i32),
    runs: &[(String, i32, i32)],
  ) -> Option<Vec<u8>> {
    let (output_width, output_height) = output;
    if output_width <= 0 || output_height <= 0 {
      return None;
    }
    let width = output_width * SUPERSAMPLE;
    let height = output_height * SUPERSAMPLE;
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
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
        Some(self.dc),
        &raw const info,
        DIB_RGB_COLORS,
        &mut bits,
        None,
        0,
      )
    }
    .ok()?;
    let previous = unsafe { SelectObject(self.dc, bitmap.into()) };
    let pixels =
      unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), (width * height * 4) as usize) };
    pixels.fill(0);
    let drawn = unsafe {
      SetBkMode(self.dc, TRANSPARENT);
      SetTextColor(self.dc, COLORREF(0x00FF_FFFF));
      runs.iter().all(|(text, x, y)| {
        let wide: Vec<u16> = text.encode_utf16().collect();
        TextOutW(self.dc, *x, *y, &wide).as_bool()
      })
    };
    let resolved = drawn.then(|| {
      let mut coverage = vec![0_u8; (output_width * output_height) as usize];
      let samples = (SUPERSAMPLE * SUPERSAMPLE) as u32;
      for y in 0..output_height {
        for x in 0..output_width {
          let mut total = 0_u32;
          for sample_y in 0..SUPERSAMPLE {
            for sample_x in 0..SUPERSAMPLE {
              let source =
                (((y * SUPERSAMPLE + sample_y) * width + x * SUPERSAMPLE + sample_x) * 4) as usize;
              total += u32::from(pixels[source + 2]);
            }
          }
          coverage[(y * output_width + x) as usize] = enhance_contrast((total / samples) as u8);
        }
      }
      coverage
    });
    unsafe {
      SelectObject(self.dc, previous);
      let _ = DeleteObject(bitmap.into());
    }
    resolved
  }
}

/// Lifts partial coverage the way DirectWrite's contrast enhancement does
/// for small text. GDI's grayscale antialiasing, box-filtered down from the
/// supersample, leaves a 14px stem at a third to a half coverage, which reads
/// as thin and grey beside the webview's text; a gamma of 1/1.6 on the
/// coverage brings the stems up without touching solid or empty pixels.
fn enhance_contrast(coverage: u8) -> u8 {
  const GAMMA: f32 = 1.0 / 1.6;
  if coverage == 0 || coverage == 255 {
    return coverage;
  }
  ((f32::from(coverage) / 255.0).powf(GAMMA) * 255.0).round() as u8
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.dc, self.previous);
      let _ = DeleteObject(self.font.into());
      let _ = DeleteDC(self.dc);
    }
  }
}
