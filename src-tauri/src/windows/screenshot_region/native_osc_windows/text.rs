// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI text rasterisation for the region OSC, the Windows twin of
//! `osc_text_texture_macos.m`. CoreText becomes the `label.rs` GDI path:
//! a bundled variable font selected by family plus `lfWeight`, rasterised at
//! twice the physical resolution and box-downsampled, because GDI's grayscale
//! grid fitting is visibly coarse at these sizes.
//!
//! Two products, exactly as on macOS:
//! * whole-string textures (Inter semibold) for chrome labels, and
//! * the fixed-cell monospace atlas (`"#0123456789ABCDEF× px≈"`, Roboto Mono
//!   semibold) the ruler assembles its readouts from.
//!
//! Both upload RGBA8 **premultiplied**: the shader un-premultiplies (`rgb/a`)
//! for kinds 11/15/37 and reads the alpha alone for the tinted chrome kind, so
//! the same texture serves either path.

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
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_MODERN, FF_SWISS, FIXED_PITCH, FW_MEDIUM, FW_SEMIBOLD,
        HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
      },
    },
  },
};

use crate::osc::geometry::{Rect, Size};

/// The 22 fixed-width cells every ruler readout is assembled from.
pub(crate) const HEX_GLYPHS: &str = "#0123456789ABCDEF× px≈";
/// One transparent column on each side of a cell, so linear filtering can
/// never bleed the neighbouring glyph in.
const GUTTER: i32 = 1;
/// Rasterisation happens at this multiple of the physical resolution and is
/// box-downsampled back, the `label.rs` precedent.
const SUPERSAMPLE: i32 = 2;
/// macOS baked near-black glyphs in light mode and white in dark mode.
const LIGHT_INK: [f32; 3] = [0.149, 0.149, 0.149];
const DARK_INK: [f32; 3] = [1.0, 1.0, 1.0];

/// Position of `glyph` in the atlas, or `None` for a character the atlas has
/// no cell for. The ruler assembles every readout out of these cells; the OCR
/// chrome only needs whole-string labels.
pub(crate) fn glyph_index(glyph: char) -> Option<usize> {
  HEX_GLYPHS.chars().position(|candidate| candidate == glyph)
}

pub(crate) fn glyph_count() -> usize {
  HEX_GLYPHS.chars().count()
}

/// Texel-centre sampling bounds for one cell: start half a texel inside the
/// gutter and stop half a texel short of the far edge, so filtering uses the
/// transparent gutter without either pulling in the next glyph or trimming
/// this one (`osc_text_texture_macos.m:198-203`).
pub(crate) fn atlas_uv(glyph_pixel_width: i32, atlas_pixel_width: i32) -> (f32, f32) {
  if atlas_pixel_width <= 0 {
    return (0.0, 0.0);
  }
  let width = f64::from(atlas_pixel_width);
  (
    ((f64::from(GUTTER) + 0.5) / width) as f32,
    (f64::from((glyph_pixel_width - 1).max(0)) / width) as f32,
  )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtlasMetrics {
  /// One cell's advance in logical points.
  pub glyph_width: f64,
  pub u_offset: f32,
  pub u_width: f32,
  pub count: usize,
}

impl AtlasMetrics {
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
  /// Present only for the monospace atlas, which the ruler indexes.
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
  mono_ink: bool,
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

impl TextCache {
  fn invalidate(&mut self, scale: f64, light_mode: bool) {
    let identity = (scale_key(scale), light_mode);
    if self.identity == Some(identity) {
      return;
    }
    self.identity = Some(identity);
    self.labels.clear();
    self.atlas = None;
  }

  /// A whole-string texture drawn as white coverage. Chrome tints it from the
  /// portable control foreground at draw time, which is what lets one texture
  /// serve the loading and error status colours the macOS `NSTextField`
  /// carried on the view.
  pub(crate) fn label(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.cached(
      device,
      text,
      scale,
      light_mode,
      font_size,
      line_height,
      false,
    )
  }

  /// The ruler's tolerance notice, the one whole-string label macOS drew with
  /// `screenwide_osc_mono_text_texture`: monospace with the ink baked in,
  /// because kind 37 un-premultiplies the sample instead of tinting it.
  pub(crate) fn ink_label(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.cached(
      device,
      text,
      scale,
      light_mode,
      font_size,
      line_height,
      true,
    )
  }

  #[allow(clippy::too_many_arguments)]
  fn cached(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
    mono_ink: bool,
  ) -> Option<Arc<TextTexture>> {
    self.invalidate(scale, light_mode);
    let key = LabelKey {
      text: text.to_owned(),
      font_size: metric_key(font_size),
      line_height: metric_key(line_height),
      mono_ink,
    };
    if let Some(cached) = self.labels.get(&key) {
      return Some(Arc::clone(cached));
    }
    let ink = if !mono_ink {
      [1.0, 1.0, 1.0]
    } else if light_mode {
      LIGHT_INK
    } else {
      DARK_INK
    };
    let texture = Arc::new(build_label(
      device,
      text,
      scale,
      font_size,
      line_height,
      ink,
      mono_ink,
    )?);
    self.labels.insert(key, Arc::clone(&texture));
    Some(texture)
  }

  /// The fixed-cell monospace atlas, with the ink colour baked in the way
  /// macOS did, because its consumers draw it with the un-premultiplying
  /// glyph kind.
  pub(crate) fn hex_atlas(
    &mut self,
    device: &ID3D11Device,
    scale: f64,
    light_mode: bool,
    font_size: f64,
    line_height: f64,
  ) -> Option<Arc<TextTexture>> {
    self.invalidate(scale, light_mode);
    if let Some(cached) = self.atlas.as_ref() {
      return Some(Arc::clone(cached));
    }
    let ink = if light_mode { LIGHT_INK } else { DARK_INK };
    let texture = Arc::new(build_atlas(device, scale, font_size, line_height, ink)?);
    self.atlas = Some(Arc::clone(&texture));
    Some(texture)
  }
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
/// chosen through `lfWeight`. GDI's semibold Inter raster is optically heavier
/// than CoreText's at toolbar sizes, so proportional chrome uses medium while
/// the fixed ruler atlas retains semibold.
struct Context {
  dc: HDC,
  font: HFONT,
  previous: HGDIOBJ,
}

impl Context {
  fn new(font_size: f64, raster_scale: f64, mono: bool) -> Option<Self> {
    if font_size <= 0.0 || raster_scale <= 0.0 {
      return None;
    }
    register_fonts();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return None;
    }
    let face: Vec<u16> = if mono { "Roboto Mono\0" } else { "Inter\0" }
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
        if mono {
          FW_SEMIBOLD.0 as i32
        } else {
          FW_MEDIUM.0 as i32
        },
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
  fn measure(&self, text: &str) -> Option<(i32, i32)> {
    let wide: Vec<u16> = text.encode_utf16().collect();
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, &wide, &mut extent) };
    (measured.as_bool() && extent.cx > 0 && extent.cy > 0).then_some((extent.cx, extent.cy))
  }

  /// Draws `runs` (text plus a raster-pixel origin) white on black and returns
  /// the box-downsampled coverage at `output` resolution. GDI antialiasing
  /// blends white glyphs into the black ground, so the red channel is plain
  /// coverage.
  fn coverage(&self, output: (i32, i32), runs: &[(String, i32, i32)]) -> Option<Vec<u8>> {
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
          coverage[(y * output_width + x) as usize] = (total / samples) as u8;
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

impl Drop for Context {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.dc, self.previous);
      let _ = DeleteObject(self.font.into());
      let _ = DeleteDC(self.dc);
    }
  }
}

/// Coverage becomes premultiplied RGBA: the shader divides by alpha again, so
/// the ink colour survives the round trip unchanged.
fn premultiply(coverage: &[u8], ink: [f32; 3]) -> Vec<u8> {
  let mut rgba = vec![0_u8; coverage.len() * 4];
  for (pixel, coverage) in rgba.chunks_exact_mut(4).zip(coverage) {
    let alpha = f32::from(*coverage);
    pixel[0] = (ink[0] * alpha).round() as u8;
    pixel[1] = (ink[1] * alpha).round() as u8;
    pixel[2] = (ink[2] * alpha).round() as u8;
    pixel[3] = *coverage;
  }
  rgba
}

fn build_label(
  device: &ID3D11Device,
  text: &str,
  scale: f64,
  font_size: f64,
  line_height: f64,
  ink: [f32; 3],
  mono: bool,
) -> Option<TextTexture> {
  if text.is_empty() || scale <= 0.0 {
    return None;
  }
  let raster_scale = scale * f64::from(SUPERSAMPLE);
  let context = Context::new(font_size, raster_scale, mono)?;
  let (extent_x, extent_y) = context.measure(text)?;
  let point_width = (f64::from(extent_x) / raster_scale).ceil().max(1.0);
  let point_height = line_height.ceil().max(1.0);
  let pixel_width = (point_width * scale).round().max(1.0) as i32;
  let pixel_height = (point_height * scale).round().max(1.0) as i32;
  // The glyph cell is centred in the line box, which is what gives every
  // control the same optical baseline as its React peer.
  let top = (pixel_height * SUPERSAMPLE - extent_y) / 2;
  let coverage = context.coverage((pixel_width, pixel_height), &[(text.to_owned(), 0, top)])?;
  drop(context);
  let view = upload(
    device,
    &premultiply(&coverage, ink),
    pixel_width,
    pixel_height,
  )?;
  Some(TextTexture {
    view,
    size: Size {
      width: point_width,
      height: point_height,
    },
    atlas: None,
  })
}

fn build_atlas(
  device: &ID3D11Device,
  scale: f64,
  font_size: f64,
  line_height: f64,
  ink: [f32; 3],
) -> Option<TextTexture> {
  if scale <= 0.0 {
    return None;
  }
  let raster_scale = scale * f64::from(SUPERSAMPLE);
  let context = Context::new(font_size, raster_scale, true)?;
  let (extent_x, extent_y) = context.measure(HEX_GLYPHS)?;
  let count = glyph_count();
  let glyph_width = (f64::from(extent_x) / raster_scale).ceil() / count as f64;
  let glyph_pixel_width = (glyph_width * scale).ceil().max(1.0) as i32;
  let cell_pixel_width = glyph_pixel_width + GUTTER * 2;
  let point_height = line_height.ceil().max(1.0);
  let pixel_height = (point_height * scale).round().max(1.0) as i32;
  let pixel_width = cell_pixel_width * count as i32;
  let top = (pixel_height * SUPERSAMPLE - extent_y) / 2;
  let runs = HEX_GLYPHS
    .chars()
    .enumerate()
    .map(|(index, glyph)| {
      (
        glyph.to_string(),
        (index as i32 * cell_pixel_width + GUTTER) * SUPERSAMPLE,
        top,
      )
    })
    .collect::<Vec<_>>();
  let coverage = context.coverage((pixel_width, pixel_height), &runs)?;
  drop(context);
  let view = upload(
    device,
    &premultiply(&coverage, ink),
    pixel_width,
    pixel_height,
  )?;
  let (u_offset, u_width) = atlas_uv(glyph_pixel_width, pixel_width);
  Some(TextTexture {
    view,
    size: Size {
      width: glyph_width * count as f64,
      height: point_height,
    },
    atlas: Some(AtlasMetrics {
      glyph_width,
      u_offset,
      u_width,
      count,
    }),
  })
}

fn upload(
  device: &ID3D11Device,
  rgba: &[u8],
  width: i32,
  height: i32,
) -> Option<ID3D11ShaderResourceView> {
  super::surface::upload_rgba(device, rgba, width as u32, height as u32)
    .inspect_err(|error| eprintln!("The Windows region OSC could not upload text: {error}"))
    .ok()
}

#[cfg(test)]
#[path = "text/tests.rs"]
mod tests;
