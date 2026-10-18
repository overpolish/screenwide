// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Type set by DirectWrite: the one text engine this backend measures and
//! rasterises annotation and caption type with, so a box is sized by the
//! same widths its text is drawn at. Annotations are Inter SemiBold;
//! captions may name any installed family.
//!
//! Not GDI: GDI stops antialiasing this face above about 116 pixels to the
//! em, and the atlas sets type at twice its drawn size, so any large box or
//! counter came out with stepped edges.

mod engine;
mod installed;

pub(crate) use installed::installed_families;

#[cfg(test)]
mod tests;

use super::face::TypeFace;
use engine::REFERENCE_SIZE;
use windows::Win32::Graphics::DirectWrite::IDWriteTextFormat3;

/// The bundled Inter face, loaded into DirectWrite's font collection.
static INTER: &[u8] = include_bytes!("../../../../assets/Inter-VariableFont_opsz,wght.ttf");

/// The face at one size in pixels.
pub struct TypeDevice {
  format: IDWriteTextFormat3,
  tabular: bool,
  ascent: f64,
  descent: f64,
}

impl TypeDevice {
  /// `face` at `size` pixels.
  pub fn in_face(face: TypeFace, size: f64) -> Result<Self, String> {
    Self::create(&face, size, false)
  }

  /// The annotations' face at `size` with tabular figures, so a counter's
  /// number does not shift as it grows. The twin of
  /// `screenwide_annotation_font(size, YES)`.
  pub fn numbers(size: f64) -> Result<Self, String> {
    Self::create(&TypeFace::ANNOTATION, size, true)
  }

  /// `face` at `size` pixels with tabular figures.
  pub fn tabular_in_face(face: TypeFace, size: f64) -> Result<Self, String> {
    Self::create(&face, size, true)
  }

  fn create(face: &TypeFace, size: f64, tabular: bool) -> Result<Self, String> {
    let size = size.max(1.0);
    engine::with(|engine| {
      let face = engine.face(face)?;
      let (ascent, descent) = (face.ascent * size, face.descent * size);
      Ok(Self {
        format: engine.format(&face, size, ascent, descent)?,
        tabular,
        ascent,
        descent,
      })
    })
  }

  /// How far `text` advances, zero for nothing to set.
  pub fn advance(&self, text: &str) -> f64 {
    advance(&self.format, text, self.tabular)
  }

  /// The face's ascent and descent at this size, in pixels.
  pub fn vertical_metrics(&self) -> (f64, f64) {
    (self.ascent, self.descent)
  }

  /// `lines` drawn into a cell of `cell` pixels, each with its top-left at
  /// the point given and its baseline the ascent below that: one coverage
  /// byte per pixel, top row first.
  pub fn draw(&self, cell: (u32, u32), lines: &[((f64, f64), &str)]) -> Result<Vec<u8>, String> {
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

/// How wide one line of type in `face` is at `font_px`. The twin of Core
/// Text's typographic width on macOS.
pub fn line_width(face: &TypeFace, line: &str, font_px: f64) -> f64 {
  if line.is_empty() || font_px.is_nan() || font_px <= 0.0 {
    return 0.0;
  }
  let reference = engine::with(|engine| Ok(engine.face(face)?.reference));
  reference.map_or(0.0, |format| {
    advance(&format, line, false) * font_px / REFERENCE_SIZE
  })
}
