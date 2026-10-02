// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation of the keyboard-shortcut strip: Windows key names set in
//! Inter, matching the React keycap, into premultiplied BGRA.

mod icons;
mod labels;
mod rasterize;
#[cfg(test)]
mod tests;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::COLORREF,
    Graphics::Gdi::{
      CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject,
      GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextCharacterExtra, SetTextColor,
      TextOutW, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DIB_RGB_COLORS, FF_SWISS, FW_NORMAL, OUT_DEFAULT_PRECIS, TRANSPARENT,
      VARIABLE_PITCH,
    },
  },
};

use super::super::keyboard_artwork::{KeyboardRaster, DESIGN_HEIGHT};
use labels::key_label;
use rasterize::rasterize_labels;

const DESIGN_MINIMUM_WIDTH: f64 = 20.0;
const DESIGN_INSET: f64 = 4.0;
const DESIGN_GAP: f64 = 4.0;
const DESIGN_RADIUS: f64 = 4.0;
const FONT_SIZE: f64 = 13.0;
const FILL_ALPHA: f64 = 0.10;
const TEXT_ALPHA: f64 = 0.85;

struct TextDevice {
  dc: windows::Win32::Graphics::Gdi::HDC,
  font: windows::Win32::Graphics::Gdi::HFONT,
  old_font: windows::Win32::Graphics::Gdi::HGDIOBJ,
}

/// The strip for the keys `codes`, in the light or dark appearance, at
/// `backing_scale` pixels per design point.
pub(crate) fn rasterize_keyboard(
  codes: &[u16],
  light: bool,
  backing_scale: f64,
) -> Result<KeyboardRaster, String> {
  let labels = codes
    .iter()
    .map(|code| key_label(*code))
    .collect::<Vec<_>>();
  rasterize_labels(&labels, light, backing_scale)
}
