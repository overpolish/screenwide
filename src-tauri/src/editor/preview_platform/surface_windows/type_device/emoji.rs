// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An emoji set in Segoe UI Emoji by DirectWrite and drawn in its own
//! colours by Direct2D into a DIB, read back as premultiplied BGRA: a
//! sticker's picture. The twin of `screenwide_emoji_draw` on macOS.
//!
//! Apart from the annotation type's engine: that one holds only Inter, in an
//! isolated factory, and reads its DIB back as coverage over black, while an
//! emoji comes from the system's own fonts and keeps its colours and its
//! transparency. Like that engine, one set for the process, behind a lock,
//! and never released.

use std::sync::{LazyLock, Mutex, PoisonError};

use windows::core::w;
use windows::Win32::{
  Foundation::RECT,
  Graphics::{
    Direct2D::{
      Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
      D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory1, ID2D1SolidColorBrush,
      D2D1_DRAW_TEXT_OPTIONS, D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
      D2D1_DRAW_TEXT_OPTIONS_NO_SNAP, D2D1_FACTORY_TYPE_SINGLE_THREADED,
      D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_SOFTWARE,
      D2D1_RENDER_TARGET_USAGE_NONE, D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
    },
    DirectWrite::{
      DWriteCreateFactory, IDWriteFactory, IDWriteTextLayout, DWRITE_FACTORY_TYPE_SHARED,
      DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
      DWRITE_WORD_WRAPPING_NO_WRAP,
    },
    Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    Gdi::{
      CreateCompatibleDC, CreateDIBSection, DeleteObject, GdiFlush, SelectObject, BITMAPINFO,
      BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC,
    },
  },
};
use windows_numerics::Vector2;

struct Engine {
  factory: IDWriteFactory,
  target: ID2D1DCRenderTarget,
  /// What a glyph the font has no colours for is filled with.
  ink: ID2D1SolidColorBrush,
  dc: HDC,
}

// Made on one thread and used from others. DirectWrite's objects are
// free-threaded, and the Direct2D ones and the context are only reached
// through the lock.
unsafe impl Send for Engine {}

static ENGINE: LazyLock<Result<Mutex<Engine>, String>> = LazyLock::new(|| {
  unsafe { Engine::new() }
    .map(Mutex::new)
    .map_err(|error| format!("Windows could not set up emoji drawing: {error}"))
});

impl Engine {
  unsafe fn new() -> windows::core::Result<Self> {
    unsafe {
      let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
      // A Direct2D 1.1 factory: colour glyphs are drawn by its targets.
      let d2d: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
      let target = d2d.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
        pixelFormat: D2D1_PIXEL_FORMAT {
          format: DXGI_FORMAT_B8G8R8A8_UNORM,
          alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        // One device-independent pixel to the atlas pixel.
        dpiX: 96.0,
        dpiY: 96.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
      })?;
      let ink = target.CreateSolidColorBrush(
        &D2D1_COLOR_F {
          r: 0.0,
          g: 0.0,
          b: 0.0,
          a: 1.0,
        },
        None,
      )?;
      let dc = CreateCompatibleDC(None);
      if dc.is_invalid() {
        return Err(windows::core::Error::from_thread());
      }
      Ok(Self {
        factory,
        target,
        ink,
        dc,
      })
    }
  }

  /// `emoji` laid out in Segoe UI Emoji at `size` pixels to the em, on one
  /// line that never wraps.
  fn layout(&self, emoji: &str, size: f64) -> windows::core::Result<IDWriteTextLayout> {
    unsafe {
      let format = self.factory.CreateTextFormat(
        w!("Segoe UI Emoji"),
        None,
        DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        size as f32,
        w!("en-us"),
      )?;
      format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
      let text: Vec<u16> = emoji.encode_utf16().collect();
      // The layout box is only an origin.
      self.factory.CreateTextLayout(&text, &format, 0.0, 0.0)
    }
  }

  unsafe fn paint(
    &self,
    cell: (u32, u32),
    origin: (f64, f64),
    layout: &IDWriteTextLayout,
  ) -> windows::core::Result<()> {
    let bounds = RECT {
      left: 0,
      top: 0,
      right: cell.0 as i32,
      bottom: cell.1 as i32,
    };
    unsafe {
      self.target.BindDC(self.dc, &bounds)?;
      self.target.BeginDraw();
      self
        .target
        .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
      self.target.Clear(Some(&D2D1_COLOR_F::default()));
      self.target.DrawTextLayout(
        Vector2 {
          X: origin.0 as f32,
          Y: origin.1 as f32,
        },
        layout,
        &self.ink,
        D2D1_DRAW_TEXT_OPTIONS(
          D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT.0 | D2D1_DRAW_TEXT_OPTIONS_NO_SNAP.0,
        ),
      );
      self.target.EndDraw(None, None)
    }
  }

  fn draw(
    &self,
    emoji: &str,
    size: f64,
    cell: (u32, u32),
    origin: (f64, f64),
  ) -> Result<Vec<u8>, String> {
    let layout = self
      .layout(emoji, size)
      .map_err(|error| format!("Windows could not lay out the emoji: {error}"))?;
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: cell.0 as i32,
        // Negative: the rows run top-down from the first pixel.
        biHeight: -(cell.1 as i32),
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
    .map_err(|error| format!("Windows could not create an emoji bitmap: {error}"))?;
    let old_bitmap = unsafe { SelectObject(self.dc, bitmap.into()) };
    let drawn = unsafe { self.paint(cell, origin, &layout) };
    let pixels = drawn.as_ref().ok().map(|()| {
      let _ = unsafe { GdiFlush() };
      let length = cell.0 as usize * cell.1 as usize * 4;
      // Premultiplied BGRA already, as the atlas holds it.
      unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), length) }.to_vec()
    });
    unsafe {
      SelectObject(self.dc, old_bitmap);
      let _ = DeleteObject(bitmap.into());
    }
    drawn.map_err(|error| format!("Windows could not draw the emoji: {error}"))?;
    Ok(pixels.unwrap_or_default())
  }
}

/// `emoji` set in Segoe UI Emoji at `size` pixels to the em into a cell of
/// `cell` pixels, its line's top-left at `origin`: premultiplied BGRA rows,
/// top row first, clear wherever the emoji is not drawn.
pub(crate) fn draw_emoji(
  emoji: &str,
  size: f64,
  cell: (u32, u32),
  origin: (f64, f64),
) -> Result<Vec<u8>, String> {
  if cell.0 == 0 || cell.1 == 0 || size.is_nan() || size <= 0.0 || emoji.is_empty() {
    return Err("There is no emoji to draw".to_owned());
  }
  let engine = ENGINE.as_ref().map_err(Clone::clone)?;
  engine
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .draw(emoji, size, cell, origin)
}
