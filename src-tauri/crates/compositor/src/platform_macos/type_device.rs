// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Inter SemiBold set by Core Text: the one text engine this backend measures
//! and rasterises annotation type with, so a box is sized by the same widths
//! its text is drawn at. The face itself is `screenwide_annotation_font`,
//! shared with the rest of the native annotation type.

#[cfg(test)]
mod tests;

/// The face at one size in pixels.
pub(crate) struct TypeDevice {
  size: f64,
  tabular: bool,
  ascent: f64,
  descent: f64,
}

#[repr(C)]
struct NativeLine {
  x: f64,
  y: f64,
  text: *const u8,
  length: u32,
}

unsafe extern "C" {
  fn screenwide_type_metrics(size: f64, tabular: u32, ascent: *mut f64, descent: *mut f64);
  fn screenwide_type_advance(text: *const u8, length: u32, size: f64, tabular: u32) -> f64;
  fn screenwide_type_draw(
    size: f64,
    tabular: u32,
    width: u32,
    height: u32,
    lines: *const NativeLine,
    count: u32,
    coverage: *mut u8,
  ) -> u32;
}

impl TypeDevice {
  /// The face at `size` pixels, its em rather than its line's height.
  pub(crate) fn new(size: f64) -> Result<Self, String> {
    Ok(Self::create(size, false))
  }

  /// The face at `size` with tabular figures, so a counter's number does not
  /// shift as it grows.
  pub(crate) fn numbers(size: f64) -> Result<Self, String> {
    Ok(Self::create(size, true))
  }

  fn create(size: f64, tabular: bool) -> Self {
    let size = size.max(1.0);
    let (mut ascent, mut descent) = (0.0, 0.0);
    unsafe { screenwide_type_metrics(size, u32::from(tabular), &mut ascent, &mut descent) };
    Self {
      size,
      tabular,
      ascent,
      descent,
    }
  }

  /// How far `text` advances, zero for nothing to set.
  pub(crate) fn advance(&self, text: &str) -> f64 {
    if text.is_empty() {
      return 0.0;
    }
    let length = u32::try_from(text.len()).unwrap_or(u32::MAX);
    unsafe { screenwide_type_advance(text.as_ptr(), length, self.size, u32::from(self.tabular)) }
      .max(0.0)
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
    let mut coverage = vec![0_u8; cell.0 as usize * cell.1 as usize];
    if coverage.is_empty() {
      return Ok(coverage);
    }
    let native = lines
      .iter()
      .filter(|(_, text)| !text.is_empty())
      .map(|((x, y), text)| NativeLine {
        x: *x,
        y: *y,
        text: text.as_ptr(),
        length: u32::try_from(text.len()).unwrap_or(u32::MAX),
      })
      .collect::<Vec<_>>();
    let drawn = unsafe {
      screenwide_type_draw(
        self.size,
        u32::from(self.tabular),
        cell.0,
        cell.1,
        native.as_ptr(),
        native.len() as u32,
        coverage.as_mut_ptr(),
      )
    };
    if drawn == 0 {
      return Err("Core Text could not draw annotation type".to_owned());
    }
    Ok(coverage)
  }
}
