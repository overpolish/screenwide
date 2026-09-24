// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Inter SemiBold set by GDI: the one text engine this backend measures and
//! rasterises annotation type with, so a box is sized by the same widths its
//! text is drawn at.

use std::cell::RefCell;

use windows::{
  core::PCWSTR,
  Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateFontW, DeleteDC, DeleteObject, GetTextExtentPoint32W,
    GetTextMetricsW, SelectObject, ANTIALIASED_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    FF_SWISS, FW_SEMIBOLD, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, TEXTMETRICW, VARIABLE_PITCH,
  },
};

/// A memory context with the face selected at one size in pixels.
pub(crate) struct TypeDevice {
  pub(crate) dc: HDC,
  font: HFONT,
  old_font: HGDIOBJ,
}

impl TypeDevice {
  /// The face at `size` pixels, its em rather than its cell height. The face
  /// is registered with GDI by the preview surface's own font module, which
  /// the keyboard artwork shares.
  pub(crate) fn new(size: f64) -> Result<Self, String> {
    super::font::register_inter_font();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return Err("Windows could not create a type drawing context".to_owned());
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
      return Err("Windows could not create the annotation font".to_owned());
    }
    let old_font = unsafe { SelectObject(dc, font.into()) };
    Ok(Self { dc, font, old_font })
  }

  /// The extent `text` is set at: its advance and the cell's height.
  pub(crate) fn measure(&self, text: &[u16]) -> Result<(i32, i32), String> {
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, text, &mut extent) };
    if !measured.as_bool() || extent.cx <= 0 || extent.cy <= 0 {
      return Err("Windows could not measure annotation type".to_owned());
    }
    Ok((extent.cx, extent.cy))
  }

  /// How far `text` advances, zero for nothing to set.
  pub(crate) fn advance(&self, text: &str) -> f64 {
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
      return 0.0;
    }
    self
      .measure(&wide)
      .map_or(0.0, |(width, _)| f64::from(width))
  }

  /// The face's ascent and descent at this size, in pixels.
  pub(crate) fn vertical_metrics(&self) -> (f64, f64) {
    let mut metrics = TEXTMETRICW::default();
    if unsafe { GetTextMetricsW(self.dc, &mut metrics) }.as_bool() {
      (f64::from(metrics.tmAscent), f64::from(metrics.tmDescent))
    } else {
      (0.0, 0.0)
    }
  }
}

impl Drop for TypeDevice {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.dc, self.old_font);
      let _ = DeleteObject(self.font.into());
      let _ = DeleteDC(self.dc);
    }
  }
}

/// The size lines are measured at before scaling to the size asked for. GDI
/// sets a face to whole pixels, so a box measured at its own small size would
/// snap its width to them; one large size scales evenly to every other.
const REFERENCE_SIZE: f64 = 256.0;

/// How wide one line of annotation type is at `font_px`. The twin of Core
/// Text's typographic width on macOS.
pub(crate) fn line_width(line: &str, font_px: f64) -> f64 {
  thread_local! {
    static REFERENCE: RefCell<Option<TypeDevice>> = const { RefCell::new(None) };
  }
  if line.is_empty() || font_px.is_nan() || font_px <= 0.0 {
    return 0.0;
  }
  REFERENCE.with(|reference| {
    let mut reference = reference.borrow_mut();
    if reference.is_none() {
      *reference = TypeDevice::new(REFERENCE_SIZE).ok();
    }
    reference.as_ref().map_or(0.0, |device| {
      device.advance(line) * font_px / REFERENCE_SIZE
    })
  })
}
