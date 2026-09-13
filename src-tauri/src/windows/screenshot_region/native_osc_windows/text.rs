// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI text rasterisation for the region OSC, the Windows twin of
//! `osc_text_texture_macos.m`. CoreText becomes the `label.rs` GDI path:
//! a bundled variable font selected by family plus `lfWeight`, rasterised at
//! twice the physical resolution and box-downsampled, because GDI's grayscale
//! grid fitting is visibly coarse at these sizes.
//!
//! Two products, exactly as on macOS:
//! * whole-string textures (Inter regular) for chrome labels, and
//! * the glyph atlas (`"#0123456789ABCDEF× px≈"`, Inter regular with tabular
//!   figures) the ruler assembles its readouts from. Its cells are one
//!   uniform width in the texture; each carries its own on-screen advance.
//!
//! Both upload RGBA8 **premultiplied**: the shader un-premultiplies (`rgb/a`)
//! for kinds 11/15/37 and reads the alpha alone for the tinted chrome kind, so
//! the same texture serves either path.

#[path = "text/cache.rs"]
mod cache;
#[path = "text/raster.rs"]
mod raster;
#[path = "text/upload.rs"]
mod upload;

use upload::{build_label, premultiply, upload};

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::{ID3D11Device, ID3D11ShaderResourceView},
      Gdi::{
        AddFontMemResourceEx, CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
        DeleteObject, GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextColor, TextOutW,
        ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_MODERN, FF_SWISS, FIXED_PITCH, FW_NORMAL, HDC, HFONT,
        HGDIOBJ, OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
      },
    },
  },
};

mod atlas;
use atlas::build_atlas;

use crate::osc::geometry::{Rect, Size};

/// The cells every ruler readout is assembled from. The texture stores them
/// on one uniform pitch, but each carries its own on-screen advance.
pub(crate) const HEX_GLYPHS: &str = "#0123456789ABCDEF× px≈";
/// `HEX_GLYPHS.chars().count()`, as an array length.
pub(crate) const ATLAS_CELLS: usize = 22;
/// One transparent column on each side of a cell, so linear filtering can
/// never bleed the neighbouring glyph in.
const GUTTER: i32 = 1;
/// Rasterisation happens at this multiple of the physical resolution and is
/// box-downsampled back, the `label.rs` precedent.
const SUPERSAMPLE: i32 = 2;
/// macOS baked near-black glyphs in light mode and white in dark mode.
/// The label tier from `src/index.css`: pure black or pure white, carried at
/// `LABEL_ALPHA`, the same colour the control foreground resolves to.
const LIGHT_INK: [f32; 3] = [0.0, 0.0, 0.0];
const DARK_INK: [f32; 3] = [1.0, 1.0, 1.0];
const LABEL_ALPHA: f32 = 0.85;

/// Position of `glyph` in the atlas, or `None` for a character the atlas has
/// no cell for. The ruler assembles every readout out of these cells; the OCR
/// chrome only needs whole-string labels.
pub(crate) fn glyph_index(glyph: char) -> Option<usize> {
  HEX_GLYPHS.chars().position(|candidate| candidate == glyph)
}

pub(crate) fn glyph_count() -> usize {
  HEX_GLYPHS.chars().count()
}

/// Cell-edge UVs map destination pixel centres to source texel centres.
/// A half-texel inset would stretch N - 1 texels across N pixels and blur.
pub(crate) fn atlas_uv(glyph_pixel_width: i32, atlas_pixel_width: i32) -> (f32, f32) {
  if atlas_pixel_width <= 0 {
    return (0.0, 0.0);
  }
  let width = f64::from(atlas_pixel_width);
  (
    (f64::from(GUTTER) / width) as f32,
    (f64::from(glyph_pixel_width.max(0)) / width) as f32,
  )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtlasMetrics {
  /// The uniform cell the texture stores each glyph in, centred: the widest
  /// advance in the set. Quads sample a whole cell, so this is the quad width.
  pub glyph_width: f64,
  /// Each cell's own advance, which is what a readout steps by on screen.
  /// Digits share one because the face is drawn with tabular figures.
  pub advances: [f64; ATLAS_CELLS],
  pub u_offset: f32,
  pub u_width: f32,
  pub count: usize,
}

impl AtlasMetrics {
  /// The advance of one cell, falling back to the uniform cell width.
  pub(crate) fn advance(&self, index: usize) -> f64 {
    self
      .advances
      .get(index)
      .copied()
      .unwrap_or(self.glyph_width)
  }

  /// The on-screen width of `text` assembled out of atlas cells.
  pub(crate) fn text_width(&self, text: &str) -> f64 {
    text
      .chars()
      .map(|glyph| self.advance(glyph_index(glyph).unwrap_or(0)))
      .sum()
  }

  /// The widest advance among `glyphs`. A field whose value changes under the
  /// pointer, such as a hex colour, is laid out on this single pitch so its
  /// columns cannot shuffle; fields that only ever hold digits and fixed
  /// separators use the per-glyph advances instead.
  pub(crate) fn pitch(&self, glyphs: &str) -> f64 {
    glyphs
      .chars()
      .map(|glyph| self.advance(glyph_index(glyph).unwrap_or(0)))
      .fold(0.0_f64, f64::max)
  }

  /// The uv rectangle of one cell. Cells are evenly spaced, so the stride is
  /// simply `1 / count` and the gutter correction rides on top.
  pub(crate) fn glyph_texture_rect(&self, index: usize) -> Rect {
    let stride = 1.0 / self.count.max(1) as f64;
    Rect::from_xywh(
      index as f64 * stride + f64::from(self.u_offset),
      0.0,
      f64::from(self.u_width),
      1.0,
    )
  }
}

pub(crate) struct TextTexture {
  pub(crate) view: ID3D11ShaderResourceView,
  /// Logical points, the size the vertex builder lays the quad out with.
  pub(crate) size: Size,
  /// Present only for the glyph atlas, which the ruler indexes.
  pub(crate) atlas: Option<AtlasMetrics>,
}

fn scale_key(scale: f64) -> u32 {
  (scale * 1000.0).round().max(0.0) as u32
}

fn metric_key(value: f64) -> u32 {
  (value * 100.0).round().max(0.0) as u32
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LabelKey {
  text: String,
  font_size: u32,
  line_height: u32,
  /// Monospace with the ink baked in, the way the atlas is drawn. Chrome
  /// labels are proportional white coverage tinted at draw time instead.
  baked_ink: bool,
}

/// Per-surface texture cache. macOS re-rasterised only when the backing scale
/// or the appearance changed, so those two are the invalidation trigger rather
/// than part of every key.
#[derive(Default)]
pub(crate) struct TextCache {
  identity: Option<(u32, bool)>,
  labels: HashMap<LabelKey, Arc<TextTexture>>,
  atlas: Option<Arc<TextTexture>>,
}

fn register_fonts() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    for font in [
      include_bytes!("../../../../assets/Inter-VariableFont_opsz,wght.ttf").as_slice(),
      include_bytes!("../../../../assets/RobotoMono-VariableFont_wght.ttf").as_slice(),
    ] {
      let mut count = 0_u32;
      let _ = unsafe {
        AddFontMemResourceEx(
          font.as_ptr().cast::<c_void>(),
          font.len() as u32,
          None,
          &raw mut count,
        )
      };
    }
  });
}

/// A memory DC with the requested face selected into it. Both assets are
/// variable fonts, so the family is selected by name and the weight axis is
/// chosen through `lfWeight`. Chrome labels and the ruler atlas are both the
/// regular body weight. Ink baking is separate from font selection.
struct Context {
  dc: HDC,
  font: HFONT,
  previous: HGDIOBJ,
}

#[cfg(test)]
#[path = "text/tests.rs"]
mod tests;
