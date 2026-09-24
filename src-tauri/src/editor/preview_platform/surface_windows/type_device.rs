// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Inter SemiBold set by DirectWrite: the one text engine this backend
//! measures and rasterises annotation type with, so a box is sized by the
//! same widths its text is drawn at.
//!
//! Not GDI: GDI stops antialiasing this face above about 116 pixels to the
//! em, and the atlas sets type at twice its drawn size, so any large box or
//! counter came out with stepped edges.

#[path = "type_device/engine.rs"]
mod engine;
#[cfg(test)]
#[path = "type_device/tests.rs"]
mod tests;

use engine::REFERENCE_SIZE;
use windows::Win32::Graphics::DirectWrite::IDWriteTextFormat3;

/// The face at one size in pixels.
pub(crate) struct TypeDevice {
  format: IDWriteTextFormat3,
  tabular: bool,
  ascent: f64,
  descent: f64,
}

impl TypeDevice {
  /// The face at `size` pixels, its em rather than its line's height.
  pub(crate) fn new(size: f64) -> Result<Self, String> {
    Self::create(size, false)
  }

  /// The face at `size` with tabular figures, so a counter's number does not
  /// shift as it grows. The twin of `screenwide_annotation_font(size, YES)`.
  pub(crate) fn numbers(size: f64) -> Result<Self, String> {
    Self::create(size, true)
  }

  fn create(size: f64, tabular: bool) -> Result<Self, String> {
    let size = size.max(1.0);
    engine::with(|engine| {
      let (ascent, descent) = (engine.ascent * size, engine.descent * size);
      Ok(Self {
        format: engine.format(size, ascent, descent)?,
        tabular,
        ascent,
        descent,
      })
    })
  }

  /// How far `text` advances, zero for nothing to set.
  pub(crate) fn advance(&self, text: &str) -> f64 {
    advance(&self.format, text, self.tabular)
  }

  /// The face's ascent and descent at this size, in pixels.
  pub(crate) fn vertical_metrics(&self) -> (f64, f64) {
    (self.ascent, self.descent)
  }

  /// `lines` drawn into a cell of `cell` pixels, each with its top-left at
  /// the point given and its baseline the ascent below that: one coverage
  /// byte per pixel, top row first.
  pub(crate) fn draw(
    &self,
    cell: (u32, u32),
    lines: &[((f64, f64), &str)],
  ) -> Result<Vec<u8>, String> {
    engine::with(|engine| {
      let layouts = lines
        .iter()
        .filter(|(_, text)| !text.is_empty())
        .map(|(at, text)| {
          let wide: Vec<u16> = text.encode_utf16().collect();
          Ok((*at, engine.layout(&self.format, &wide, self.tabular)?))
        })
        .collect::<Result<Vec<_>, String>>()?;
      engine.draw(cell, &layouts)
    })
  }
}

/// How far `text` advances set in `format`, zero for nothing to set.
fn advance(format: &IDWriteTextFormat3, text: &str, tabular: bool) -> f64 {
  if text.is_empty() {
    return 0.0;
  }
  let wide: Vec<u16> = text.encode_utf16().collect();
  engine::with(|engine| engine.layout(format, &wide, tabular))
    .map_or(0.0, |layout| engine::width(&layout))
}

/// How wide one line of annotation type is at `font_px`. The twin of Core
/// Text's typographic width on macOS.
pub(crate) fn line_width(line: &str, font_px: f64) -> f64 {
  if line.is_empty() || font_px.is_nan() || font_px <= 0.0 {
    return 0.0;
  }
  let reference = engine::with(|engine| Ok(engine.reference.clone()));
  reference.map_or(0.0, |format| {
    advance(&format, line, false) * font_px / REFERENCE_SIZE
  })
}
