// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a line's words are, read off the picture itself.
//!
//! A reader's boxes for the parts of a line are not always where the parts
//! are: Vision, given a row of cells it read as one line, answers every
//! character with an even share of the whole line, as if the gaps between
//! the cells were not there. The picture still has them. Across the line's
//! box, each column of pixels either has ink, anything unlike the line's
//! background, or not; runs of inked columns a letter's gap apart are a
//! word, and the gaps between them are where the words are apart.

use crate::Rect;

/// A picture, four bytes a pixel, rows top to bottom.
#[derive(Clone, Copy)]
pub struct Picture<'a> {
  pub rgba: &'a [u8],
  pub width: u32,
  pub height: u32,
}

/// How far a pixel's grey, out of 255, is from the background to be ink:
/// past the softening at a letter's edge.
const INK_LEVEL: i32 = 56;
/// The widest gap inside a word, as a share of the line's height: letters'
/// gaps are narrower, and a space is wider.
const LETTER_GAP: f64 = 0.12;
/// A run at most this wide, as a share of the line's height, that inks most
/// of the line's rows is a table's rule, not a letter.
const RULE_WIDTH: f64 = 0.1;
const RULE_ROWS: f64 = 0.85;

/// The runs of ink across `bounds`, a share of the picture, from start to
/// end, in the picture's shares; none where the box is off the picture or
/// too small to read.
pub(crate) fn ink_runs(picture: Picture, bounds: Rect) -> Vec<(f64, f64)> {
  let (width, height) = (f64::from(picture.width), f64::from(picture.height));
  let left = (bounds.x * width).floor().max(0.0) as usize;
  let right = ((bounds.x + bounds.width) * width).ceil().min(width) as usize;
  let top = (bounds.y * height).floor().max(0.0) as usize;
  let bottom = ((bounds.y + bounds.height) * height).ceil().min(height) as usize;
  let stride = picture.width as usize * 4;
  if right <= left + 1 || bottom <= top + 2 || picture.rgba.len() < bottom * stride {
    return Vec::new();
  }
  let grey = |x: usize, y: usize| {
    let pixel = &picture.rgba[y * stride + x * 4..][..3];
    (i32::from(pixel[0]) * 77 + i32::from(pixel[1]) * 150 + i32::from(pixel[2]) * 29) >> 8
  };
  let mut counts = [0_u32; 32];
  for y in top..bottom {
    for x in left..right {
      counts[(grey(x, y) >> 3) as usize] += 1;
    }
  }
  let background = counts
    .iter()
    .enumerate()
    .max_by_key(|&(_, count)| *count)
    .map_or(0, |(bucket, _)| (bucket as i32) * 8 + 4);
  let rows = bottom - top;
  let inked: Vec<usize> = (left..right)
    .map(|x| {
      (top..bottom)
        .filter(|&y| (grey(x, y) - background).abs() > INK_LEVEL)
        .count()
    })
    .collect();
  let line_height = rows as f64;
  let letter_gap = (LETTER_GAP * line_height).max(1.0) as usize;
  let rule_width = (RULE_WIDTH * line_height).max(2.0) as usize;
  let mut runs: Vec<(usize, usize)> = Vec::new();
  let mut x = 0;
  while x < inked.len() {
    if inked[x] == 0 {
      x += 1;
      continue;
    }
    let start = x;
    let mut tallest = 0;
    while x < inked.len() && inked[x] > 0 {
      tallest = tallest.max(inked[x]);
      x += 1;
    }
    if x - start <= rule_width && tallest as f64 >= RULE_ROWS * line_height {
      continue;
    }
    match runs.last_mut() {
      Some(last) if start - last.1 <= letter_gap => last.1 = x,
      _ => runs.push((start, x)),
    }
  }
  runs
    .into_iter()
    .map(|(start, end)| ((left + start) as f64 / width, (left + end) as f64 / width))
    .collect()
}
