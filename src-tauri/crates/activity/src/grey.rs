// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shrinking a captured frame to the small grey picture that is measured,
//! straight from the capture's own pixels: BGRA from a screen, or the luma
//! plane of a camera's YUV.

use crate::measure::SAMPLE_SIZE;

/// Points read across each cell, along each side: a 4 by 4 lattice. Enough
/// that a dialog, a scroll or a page change moves a cell's average, at a
/// sixteenth of reading every pixel of a 3600-wide screen.
const POINTS: usize = 4;

/// `pixels`, BGRA rows `stride` bytes apart, as `SAMPLE_SIZE` grey cells,
/// each the average of a lattice of points spread across it.
pub fn grey_from_bgra(pixels: &[u8], width: usize, height: usize, stride: usize) -> Vec<u8> {
  cells(pixels, (width, height, stride, 4), |pixel| {
    // Luma, weighted as the eye weighs the three.
    (u32::from(pixel[2]) * 77 + u32::from(pixel[1]) * 150 + u32::from(pixel[0]) * 29) >> 8
  })
}

/// A luma plane, one byte a pixel and rows `stride` bytes apart, as
/// `SAMPLE_SIZE` grey cells.
pub fn grey_from_luma(plane: &[u8], width: usize, height: usize, stride: usize) -> Vec<u8> {
  cells(plane, (width, height, stride, 1), |pixel| {
    u32::from(pixel[0])
  })
}

fn cells(
  pixels: &[u8],
  (width, height, stride, size): (usize, usize, usize, usize),
  grey: impl Fn(&[u8]) -> u32,
) -> Vec<u8> {
  let (columns, rows) = SAMPLE_SIZE;
  if width == 0 || height == 0 || pixels.len() < stride * (height - 1) + width * size {
    return vec![0; columns * rows];
  }
  let mut out = Vec::with_capacity(columns * rows);
  for row in 0..rows {
    for column in 0..columns {
      let mut sum = 0;
      for step_y in 0..POINTS {
        // Points sit in the middle of each of the cell's sub-squares, so a
        // cell smaller than the lattice still reads inside itself.
        let y = ((row * POINTS + step_y) * 2 + 1) * height / (rows * POINTS * 2);
        for step_x in 0..POINTS {
          let x = ((column * POINTS + step_x) * 2 + 1) * width / (columns * POINTS * 2);
          let at = y * stride + x * size;
          sum += grey(&pixels[at..at + size]);
        }
      }
      out.push((sum / (POINTS * POINTS) as u32) as u8);
    }
  }
  out
}
