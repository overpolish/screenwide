// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How much room a block of text takes at a type size.
//!
//! A box is its text plus padding, so the box can only be known once the text
//! has been measured, and only the text engine that draws the type can say
//! how wide it is. The platform measures; everything that places, picks or
//! snaps a box asks here, so the drawing and the hand agree on one size.
//!
//! The block is laid out line by line - a line break is the only thing that
//! starts a new line - at a fixed line height, so the height is exact and the
//! same on every platform. Only the widths are the engine's.

use std::cell::RefCell;
use std::collections::HashMap;

use super::face::TypeFace;

/// A line's height, in ems. The twin of `SCREENWIDE_TEXT_BOX_LINE_HEIGHT` in
/// `macos/annotation_text_box.h`; the shared annotation atlas
/// reads it here.
pub const LINE_HEIGHT: f64 = 1.25;

/// The block of `text` at `font_px`: its widest line and its line count times
/// the line height, in the pixels `font_px` is in. An empty text is one empty
/// line, which is where the caret stands in a fresh box.
pub fn text_block(text: &str, font_px: f64) -> [f64; 2] {
  face_text_block(&TypeFace::ANNOTATION, text, font_px)
}

/// [`text_block`] set in `face` rather than the annotations' Inter.
pub fn face_text_block(face: &TypeFace, text: &str, font_px: f64) -> [f64; 2] {
  if !font_px.is_finite() || font_px <= 0.0 {
    return [0.0; 2];
  }
  let lines = text.split('\n').count() as f64;
  [
    widest_line(face, text, font_px),
    lines * LINE_HEIGHT * font_px,
  ]
}

/// How wide one line of type in `face` is at `font_px`, as the platform's
/// text engine sets it.
pub fn line_width(face: &TypeFace, line: &str, font_px: f64) -> f64 {
  platform::line_width(face, line, font_px)
}

/// The widest line's width, measured once per face, text and size. A frame
/// asks for every box it draws, the chrome for every box it picks and the
/// snap engine for every box it lines up against, so the answers are kept;
/// the cache is dropped whole once it grows, which a document never
/// approaches, and when the templates bring fonts, which may set a family
/// measured before in another face.
fn widest_line(face: &TypeFace, text: &str, font_px: f64) -> f64 {
  const LIMIT: usize = 512;
  /// The fonts' generation the widths were measured in, and each width by
  /// face, text and size.
  type Widths = (u32, HashMap<(TypeFace, String, u64), f64>);
  thread_local! {
    static WIDTHS: RefCell<Widths> = RefCell::new((0, HashMap::new()));
  }
  let generation = super::app_fonts::app_fonts_generation();
  let key = (face.clone(), text.to_owned(), font_px.to_bits());
  let known = WIDTHS.with(|widths| {
    let widths = widths.borrow();
    (widths.0 == generation)
      .then(|| widths.1.get(&key).copied())
      .flatten()
  });
  if let Some(width) = known {
    return width;
  }
  let width = text
    .split('\n')
    .map(|line| platform::line_width(face, line, font_px))
    .fold(0.0, f64::max);
  WIDTHS.with(|widths| {
    let mut widths = widths.borrow_mut();
    if widths.0 != generation || widths.1.len() >= LIMIT {
      *widths = (generation, HashMap::new());
    }
    widths.1.insert(key, width);
  });
  width
}

#[cfg(target_os = "macos")]
mod platform {
  use super::TypeFace;

  extern "C" {
    /// Core Text's typographic width of one line in a face, in
    /// `macos/annotation_text_box.m`.
    fn screenwide_face_line_width(
      text: *const u8,
      length: u32,
      font: f64,
      family: *const u8,
      family_length: u32,
      weight: u32,
      italic: u32,
    ) -> f64;
  }

  pub(super) fn line_width(face: &TypeFace, line: &str, font_px: f64) -> f64 {
    if line.is_empty() {
      return 0.0;
    }
    let length = u32::try_from(line.len()).unwrap_or(u32::MAX);
    let family = face.family_bytes();
    // Safety: each pointer and length describe a string that outlives the
    // call.
    unsafe {
      screenwide_face_line_width(
        line.as_ptr(),
        length,
        font_px,
        family.as_ptr(),
        family.len() as u32,
        u32::from(face.weight),
        u32::from(face.italic),
      )
    }
    .max(0.0)
  }
}

#[cfg(target_os = "windows")]
mod platform {
  use super::TypeFace;

  pub(super) fn line_width(face: &TypeFace, line: &str, font_px: f64) -> f64 {
    crate::engine::type_device::line_width(face, line, font_px)
  }
}

/// Test builds off both platforms have no text engine to ask; a line is
/// sized from its character count at Inter's average advance.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  use super::TypeFace;

  pub(super) fn line_width(_face: &TypeFace, line: &str, font_px: f64) -> f64 {
    line.chars().count() as f64 * font_px * 0.56
  }
}
