// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Laid-out lines drawn by Direct2D into a DIB the engine's context holds,
//! and read back as coverage.

use windows::Win32::{
  Foundation::RECT,
  Graphics::{
    Direct2D::{
      Common::D2D1_COLOR_F, D2D1_DRAW_TEXT_OPTIONS_NO_SNAP, D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
    },
    DirectWrite::IDWriteTextLayout,
    Gdi::{
      CreateDIBSection, DeleteObject, GdiFlush, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
      DIB_RGB_COLORS,
    },
  },
};
use windows_numerics::Vector2;

use super::Engine;

impl Engine {
  /// `lines`, each with its layout box's top-left at the point given, drawn
  /// white over black into a cell of `cell` pixels: one coverage byte per
  /// pixel, top row first.
  pub(in super::super) fn draw(
    &self,
    cell: (u32, u32),
    lines: &[((f64, f64), IDWriteTextLayout)],
  ) -> Result<Vec<u8>, String> {
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: cell.0 as i32,
        // Negative: the rows run top-down from the first pixel, which is the
        // space the shader samples the rectangles in.
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
    .map_err(|error| format!("Windows could not create a type bitmap: {error}"))?;
    let old_bitmap = unsafe { SelectObject(self.dc, bitmap.into()) };
    let drawn = unsafe { self.paint(cell, lines) };
    let coverage = drawn.as_ref().ok().map(|()| {
      let _ = unsafe { GdiFlush() };
      let length = cell.0 as usize * cell.1 as usize * 4;
      let raster = unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), length) };
      // White type over black: any one channel is its coverage.
      raster.chunks_exact(4).map(|pixel| pixel[2]).collect()
    });
    unsafe {
      SelectObject(self.dc, old_bitmap);
      let _ = DeleteObject(bitmap.into());
    }
    drawn.map_err(|error| format!("Windows could not draw annotation type: {error}"))?;
    Ok(coverage.unwrap_or_default())
  }

  unsafe fn paint(
    &self,
    cell: (u32, u32),
    lines: &[((f64, f64), IDWriteTextLayout)],
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
      self.target.SetTextRenderingParams(&self.params);
      self.target.Clear(Some(&D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
      }));
      for ((x, y), layout) in lines {
        // Unsnapped, so the type sits exactly where the caret and the
        // selection were measured to.
        self.target.DrawTextLayout(
          Vector2 {
            X: *x as f32,
            Y: *y as f32,
          },
          layout,
          &self.ink,
          D2D1_DRAW_TEXT_OPTIONS_NO_SNAP,
        );
      }
      self.target.EndDraw(None, None)
    }
  }
}
