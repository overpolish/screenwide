// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Type set by Core Text: the one text engine this backend measures and
//! rasterises annotation and caption type with, so a box is sized by the
//! same widths its text is drawn at. The faces themselves are
//! `screenwide_annotation_font` and `screenwide_face_font`, shared with the
//! rest of the native annotation type.

use screenwide_annotations::text::face::TypeFace;

#[cfg(test)]
mod tests;

/// A face at one size in pixels.
pub(crate) struct TypeDevice {
  face: TypeFace,
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

/// The twin of `ScreenwideTypeFace` in `type_device_macos.m`.
#[repr(C)]
struct NativeFace {
  family: *const u8,
  family_length: u32,
  weight: u32,
  italic: u32,
  tabular: u32,
}

unsafe extern "C" {
  fn screenwide_type_metrics(
    face: *const NativeFace,
    size: f64,
    ascent: *mut f64,
    descent: *mut f64,
  );
  fn screenwide_type_advance(
    face: *const NativeFace,
    text: *const u8,
    length: u32,
    size: f64,
  ) -> f64;
  fn screenwide_type_draw(
    face: *const NativeFace,
    size: f64,
    width: u32,
    height: u32,
    lines: *const NativeLine,
    count: u32,
    coverage: *mut u8,
  ) -> u32;
}

impl TypeDevice {
  /// `face` at `size` pixels.
  pub(crate) fn in_face(face: TypeFace, size: f64) -> Result<Self, String> {
    Ok(Self::create(face, size, false))
  }

  /// The annotations' face at `size` with tabular figures, so a counter's
  /// number does not shift as it grows.
  pub(crate) fn numbers(size: f64) -> Result<Self, String> {
    Ok(Self::create(TypeFace::ANNOTATION, size, true))
  }

  fn create(face: TypeFace, size: f64, tabular: bool) -> Self {
    let size = size.max(1.0);
    let mut device = Self {
      face,
      size,
      tabular,
      ascent: 0.0,
      descent: 0.0,
    };
    let (mut ascent, mut descent) = (0.0, 0.0);
    // Safety: the face points into `device`, which outlives the call.
    unsafe { screenwide_type_metrics(&device.native(), size, &mut ascent, &mut descent) };
    device.ascent = ascent;
    device.descent = descent;
    device
  }

  /// The face as the native side reads it, pointing into `self`.
  fn native(&self) -> NativeFace {
    let family = self.face.family_bytes();
    NativeFace {
      family: family.as_ptr(),
      family_length: family.len() as u32,
      weight: u32::from(self.face.weight),
      italic: u32::from(self.face.italic),
      tabular: u32::from(self.tabular),
    }
  }

  /// How far `text` advances, zero for nothing to set.
  pub(crate) fn advance(&self, text: &str) -> f64 {
    if text.is_empty() {
      return 0.0;
    }
    let length = u32::try_from(text.len()).unwrap_or(u32::MAX);
    // Safety: the face and the text outlive the call.
    unsafe { screenwide_type_advance(&self.native(), text.as_ptr(), length, self.size) }.max(0.0)
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
    // Safety: the face, the lines and the coverage outlive the call, and the
    // coverage is the cell's size.
    let drawn = unsafe {
      screenwide_type_draw(
        &self.native(),
        self.size,
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
