// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How large a classically pixelated redaction's cells are, and how wide a
//! blurred one's blur.
//!
//! A classically pixelated box is cut into square cells, centred on it, and
//! each is painted flat in its average, which the compositor's cells pass
//! reads from the pixels themselves, so the same boxes redact a screenshot
//! and every frame of a recording. A blur is a Gaussian over the box's own
//! pixels, which can widen smoothly as it arrives. Both are sized in logical
//! points of the captured content, so they look as strong in a small box as
//! in a large one, and alike on a 2x capture and a 1x one.

/// A blur's standard deviation at each strength, weakest first, in logical
/// points: about as soft as the averaged cells it replaced. The twin of
/// `ANNOTATION_BLUR_STRENGTHS` in
/// `src/components/shared/annotation-style/widths.ts`, which names the steps.
const BLUR_DEVIATIONS: [f64; 5] = [4.5, 6.0, 7.5, 10.0, 13.0];

/// The smallest block classic pixelation draws, in logical points: anything
/// finer reads as a picture rather than as pixelation. The twin of the first
/// of `ANNOTATION_CLASSIC_SIZES` in `widths.ts`.
const MIN_CLASSIC_BLOCK: f64 = 8.0;

/// A blurred box's standard deviation, in source pixels, for `strength` from
/// 1 to 5, where one logical point is `source_per_point` source pixels.
pub(crate) fn blur_deviation(strength: f64, source_per_point: f64) -> f64 {
  let step = if strength.is_finite() {
    strength.round().clamp(1.0, BLUR_DEVIATIONS.len() as f64) as usize - 1
  } else {
    0
  };
  BLUR_DEVIATIONS[step] * source_per_point
}

/// The side of a classically pixelated box's block, in source pixels: the
/// `block` asked for in logical points, and never less than the smallest
/// classic block.
pub(crate) fn mosaic_cell(block: f64, source_per_point: f64) -> f64 {
  let block = if block.is_finite() {
    block
  } else {
    MIN_CLASSIC_BLOCK
  };
  (block.max(MIN_CLASSIC_BLOCK) * source_per_point).max(1.0)
}

#[cfg(test)]
mod tests;
