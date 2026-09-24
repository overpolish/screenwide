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

/// A line's height, in ems. The twin of `SCREENWIDE_TEXT_BOX_LINE_HEIGHT` in
/// `cursor_export/gpu_compositor_macos_annotation_text.h`; the D3D11 atlas
/// reads it here.
pub(crate) const LINE_HEIGHT: f64 = 1.25;

/// The block of `text` at `font_px`: its widest line and its line count times
/// the line height, in the pixels `font_px` is in. An empty text is one empty
/// line, which is where the caret stands in a fresh box.
pub(crate) fn text_block(text: &str, font_px: f64) -> [f64; 2] {
  if !font_px.is_finite() || font_px <= 0.0 {
    return [0.0; 2];
  }
  let lines = text.split('\n').count() as f64;
  [widest_line(text, font_px), lines * LINE_HEIGHT * font_px]
}

/// The widest line's width, measured once per text and size. A frame asks for
/// every box it draws, the chrome for every box it picks and the snap engine
/// for every box it lines up against, so the answers are kept; the cache is
/// dropped whole once it grows, which a document never approaches.
fn widest_line(text: &str, font_px: f64) -> f64 {
  const LIMIT: usize = 512;
  thread_local! {
    static WIDTHS: RefCell<HashMap<(String, u64), f64>> = RefCell::new(HashMap::new());
  }
  let key = (text.to_owned(), font_px.to_bits());
  if let Some(width) = WIDTHS.with(|widths| widths.borrow().get(&key).copied()) {
    return width;
  }
  let width = text
    .split('\n')
    .map(|line| platform::line_width(line, font_px))
    .fold(0.0, f64::max);
  WIDTHS.with(|widths| {
    let mut widths = widths.borrow_mut();
    if widths.len() >= LIMIT {
      widths.clear();
    }
    widths.insert(key, width);
  });
  width
}

#[cfg(target_os = "macos")]
mod platform {
  extern "C" {
    /// Core Text's typographic width of one line of Inter SemiBold, in
    /// `gpu_compositor_macos_annotation_text_box.m`.
    fn screenwide_text_box_line_width(text: *const u8, length: u32, font: f64) -> f64;
  }

  pub(super) fn line_width(line: &str, font_px: f64) -> f64 {
    if line.is_empty() {
      return 0.0;
    }
    let length = u32::try_from(line.len()).unwrap_or(u32::MAX);
    // Safety: the pointer and length describe `line`, which outlives the call.
    unsafe { screenwide_text_box_line_width(line.as_ptr(), length, font_px) }.max(0.0)
  }
}

#[cfg(target_os = "windows")]
mod platform {
  pub(super) fn line_width(line: &str, font_px: f64) -> f64 {
    crate::editor::preview_platform::type_device::line_width(line, font_px)
  }
}

/// Test builds off both platforms have no text engine to ask; a line is
/// sized from its character count at Inter's average advance.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  pub(super) fn line_width(line: &str, font_px: f64) -> f64 {
    line.chars().count() as f64 * font_px * 0.56
  }
}
