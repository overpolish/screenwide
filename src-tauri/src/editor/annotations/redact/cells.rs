// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How large the cells of a blurred or classically pixelated redaction are.
//!
//! The box is cut into square cells, centred on it, and each is reduced to
//! its average colour by the compositor's cells pass, which reads the pixels
//! themselves, so the same boxes redact a screenshot and every frame of a
//! recording. Classic pixelation paints each cell flat in its average, which
//! is ordinary pixelation and all it shows. A blur paints a smooth surface
//! through the averages, each nudged a little by the seed, so nothing finer
//! than a cell survives. Both are sized in output pixels, so a blur looks as
//! strong in a small box as in a large one.

/// A blur's cell at each strength, weakest first, in output pixels. The
/// twin of `ANNOTATION_BLUR_STRENGTHS` in
/// `src/components/shared/annotation-style/widths.ts`, which names the steps.
const BLUR_CELLS: [f64; 5] = [14.0, 18.0, 24.0, 32.0, 40.0];

/// The smallest block classic pixelation draws, in output pixels: anything
/// finer reads as a picture rather than as pixelation. The twin of the first
/// of `ANNOTATION_CLASSIC_SIZES` in `widths.ts`.
const MIN_CLASSIC_BLOCK: f64 = 16.0;

/// The side of a blurred box's cell, in source pixels, for `strength` from 1
/// to 5, where one output pixel is `source_per_output` source pixels.
pub(crate) fn blur_cell(strength: f64, source_per_output: f64) -> f64 {
  let step = if strength.is_finite() {
    strength.round().clamp(1.0, BLUR_CELLS.len() as f64) as usize - 1
  } else {
    0
  };
  (BLUR_CELLS[step] * source_per_output).max(1.0)
}

/// The side of a classically pixelated box's block, in source pixels: the
/// `block` asked for in output pixels, and never less than the smallest
/// classic block.
pub(crate) fn mosaic_cell(block: f64, source_per_output: f64) -> f64 {
  let block = if block.is_finite() {
    block
  } else {
    MIN_CLASSIC_BLOCK
  };
  (block.max(MIN_CLASSIC_BLOCK) * source_per_output).max(1.0)
}

#[cfg(test)]
#[path = "cells_tests.rs"]
mod tests;
