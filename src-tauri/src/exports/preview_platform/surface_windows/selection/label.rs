// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation and D3D11 upload for selection-overlay labels.

use std::{ffi::c_void, sync::OnceLock};

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::ID3D11Device,
      Gdi::{
        AddFontMemResourceEx, CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
        DeleteObject, GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextColor, TextOutW,
        ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_MODERN, FF_SWISS, FIXED_PITCH, FW_MEDIUM, FW_SEMIBOLD,
        OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
      },
    },
  },
};

use super::label_texture::{upload_label_texture, LabelTexture};

/// Point size of the size readout's monospaced font, as on macOS.
const LABEL_FONT_SIZE: f64 = 11.0;
/// React's compact Button uses `text-xs` (12px) and `font-semibold`.
const ACTION_FONT_SIZE: f64 = 12.0;
/// Width of the halo stroke in points; it is centred on the glyph outline, so
/// half of it spills outside the glyph and is what the shader dilates by.
pub(super) const LABEL_STROKE: f64 = 2.0;
/// Transparent padding around the glyph box in points, room for the halo.
const LABEL_PADDING: f64 = 2.0;

pub(super) fn label_scale_key(scale: f64) -> u32 {
  (scale * 1000.0).round().max(0.0) as u32
}

pub(in crate::exports::preview_platform::surface) fn register_inter_font() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    static FONT: &[u8] = include_bytes!("../../../../../assets/Inter-VariableFont_opsz,wght.ttf");
    let mut fonts = 0u32;
    let _ = unsafe {
      AddFontMemResourceEx(
        FONT.as_ptr().cast::<c_void>(),
        FONT.len() as u32,
        None,
        &raw mut fonts,
      )
    };
  });
}

/// Rasterises `text` with GDI as white-on-black grayscale coverage: a
/// appropriately weighted face scaled by the display scale,
/// padded by `LABEL_PADDING` points on every side. Dimension readouts remain
/// monospaced; action text uses the app's proportional UI family. The red
/// channel of the returned BGRA bitmap is the glyph coverage.
fn rasterize_label(text: &str, scale: f64, action: bool) -> Result<(Vec<u8>, (u32, u32)), String> {
  let wide: Vec<u16> = text.encode_utf16().collect();
  // GDI's grayscale grid fitting is visibly coarse for 12px semibold text.
  // Rasterise action labels at twice the physical resolution and resolve the
  // coverage back to the actual texture size. This approximates DirectWrite's
  // natural antialiasing without colour fringes, which would be incorrect as
  // the opaque button colour animates underneath the label.
  let raster_factor = if action { 2 } else { 1 };
  let raster_scale = scale * f64::from(raster_factor);
  let dc = unsafe { CreateCompatibleDC(None) };
  if dc.is_invalid() {
    return Err("Windows could not create a label drawing context".to_owned());
  }
  if action {
    register_inter_font();
  }
  // Use the same family and weight as the web UI. This asset is a variable
  // font: `Inter SemiBold` is not a real face name in it, and asking GDI for
  // that name can select a substituted or synthesized static face with much
  // heavier grid fitting at text-xs sizes. Select the family and let lfWeight
  // choose the variable font's 600 axis instead.
  let face: Vec<u16> = if action { "Inter\0" } else { "Consolas\0" }
    .encode_utf16()
    .collect();
  let family = if action {
    VARIABLE_PITCH.0 | FF_SWISS.0
  } else {
    FIXED_PITCH.0 | FF_MODERN.0
  };
  let font = unsafe {
    let font_size = if action {
      ACTION_FONT_SIZE
    } else {
      LABEL_FONT_SIZE
    };
    let font_weight = if action { FW_SEMIBOLD } else { FW_MEDIUM };
    CreateFontW(
      -((font_size * raster_scale).round() as i32).max(1),
      0,
      0,
      0,
      font_weight.0 as i32,
      0,
      0,
      0,
      DEFAULT_CHARSET,
      OUT_DEFAULT_PRECIS,
      CLIP_DEFAULT_PRECIS,
      ANTIALIASED_QUALITY,
      family as u32,
      PCWSTR(face.as_ptr()),
    )
  };
  if font.is_invalid() {
    let _ = unsafe { DeleteDC(dc) };
    return Err("Windows could not create the label font".to_owned());
  }
  let old_font = unsafe { SelectObject(dc, font.into()) };
  let mut extent = Default::default();
  let measured = unsafe { GetTextExtentPoint32W(dc, &wide, &mut extent) };
  if !measured.as_bool() || extent.cx <= 0 || extent.cy <= 0 {
    unsafe {
      SelectObject(dc, old_font);
      let _ = DeleteObject(font.into());
      let _ = DeleteDC(dc);
    }
    return Err("Windows could not measure the label text".to_owned());
  }
  let padding = (LABEL_PADDING * scale).ceil() as i32;
  let output_width = ((extent.cx + raster_factor - 1) / raster_factor + padding * 2).max(1);
  // React text-xs has a 16px line box; GDI reports only the glyph cell. Give
  // actions that same line box before the button's py-1 is applied.
  let output_height = if action {
    // The web button's text-xs line box is fixed at 16px. GDI includes
    // internal leading in `extent.cy` (and reports proportionally more at the
    // supersampled size), so using that metric would inflate the compact
    // button even though the glyph ink fits the intended line box.
    ((16.0 * scale).round() as i32).max(1)
  } else {
    (extent.cy + padding * 2).max(1)
  };
  let width = output_width * raster_factor;
  let height = output_height * raster_factor;
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
  let bitmap = match unsafe {
    CreateDIBSection(
      Some(dc),
      &raw const info,
      DIB_RGB_COLORS,
      &mut bits,
      None,
      0,
    )
  } {
    Ok(bitmap) => bitmap,
    Err(error) => {
      unsafe {
        SelectObject(dc, old_font);
        let _ = DeleteObject(font.into());
        let _ = DeleteDC(dc);
      }
      return Err(error.to_string());
    }
  };
  let old_bitmap = unsafe { SelectObject(dc, bitmap.into()) };
  let length = (width * height * 4) as usize;
  let pixels = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
  // Black, opaque background: GDI antialiasing blends the white glyphs into
  // whatever is underneath, so the red channel ends up as plain coverage.
  for pixel in pixels.chunks_exact_mut(4) {
    pixel.fill(0);
    pixel[3] = 255;
  }
  let drawn = unsafe {
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, COLORREF(0x00FF_FFFF));
    let y = if action {
      (height - extent.cy) / 2
    } else {
      padding
    };
    TextOutW(dc, padding * raster_factor, y, &wide)
  };
  let result = if drawn.as_bool() {
    if raster_factor == 1 {
      Ok((pixels.to_vec(), (width as u32, height as u32)))
    } else {
      let mut resolved = vec![0u8; (output_width * output_height * 4) as usize];
      for y in 0..output_height {
        for x in 0..output_width {
          let mut coverage = 0u32;
          for sample_y in 0..raster_factor {
            for sample_x in 0..raster_factor {
              let source = (((y * raster_factor + sample_y) * width + x * raster_factor + sample_x)
                * 4) as usize;
              coverage += u32::from(pixels[source + 2]);
            }
          }
          let destination = ((y * output_width + x) * 4) as usize;
          let coverage = (coverage / (raster_factor * raster_factor) as u32) as u8;
          resolved[destination..destination + 3].fill(coverage);
          resolved[destination + 3] = 255;
        }
      }
      Ok((resolved, (output_width as u32, output_height as u32)))
    }
  } else {
    Err("Windows could not draw the label text".to_owned())
  };
  unsafe {
    SelectObject(dc, old_bitmap);
    SelectObject(dc, old_font);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteObject(font.into());
    let _ = DeleteDC(dc);
  }
  result
}

pub(super) fn build_label_texture(
  device: &ID3D11Device,
  text: &str,
  scale: f64,
  action: bool,
) -> Result<LabelTexture, String> {
  let (pixels, size) = rasterize_label(text, scale, action)?;
  upload_label_texture(device, &pixels, size, text, label_scale_key(scale), action)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_label_resolves_supersampled_grayscale_coverage() {
    let (pixels, (width, height)) = rasterize_label("Recenter", 1.0, true).unwrap();

    assert_eq!(height, 16);
    assert_eq!(pixels.len(), (width * height * 4) as usize);
    assert!(pixels
      .chunks_exact(4)
      .any(|pixel| pixel[2] > 0 && pixel[2] < 255));
    assert!(pixels
      .chunks_exact(4)
      .all(|pixel| pixel[0] == pixel[1] && pixel[1] == pixel[2] && pixel[3] == 255));
  }
}
