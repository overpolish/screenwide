// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A stroke's retained draw record.
//!
//! `p0` is its box's top-left corner, `p1` that corner moved one source pixel
//! across and down, and `p2` the box's bottom-right corner: every placement
//! carries all three into the pixels the stroke is drawn in, and the shader
//! places the fitted chain by them. The chain follows in the side buffer's
//! points, `data_count` of them from `data_offset`, each from the box's
//! top-left corner in source pixels. After it comes one entry a curve: where
//! along the stroke that curve starts and ends, as shares of its length, for
//! the shader to draw a stroke drawing itself in by its record's `low` and
//! `high`. `params[2]` is that length in source pixels, which the exposure
//! reads the way it reads a highlight's sweep: what the drawing end covers
//! over the whole reveal.

use super::model::bounds;
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::AnnotationPoint;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn draw_points(points: &[AnnotationPoint]) -> [[f32; 2]; 3] {
  let (low, high) = bounds(points);
  let low = [low.x as f32, low.y as f32];
  [
    low,
    [low[0] + 1.0, low[1] + 1.0],
    [high.x as f32, high.y as f32],
  ]
}

/// Writes the fitted chain, and where along the stroke each of its curves
/// lies, into `side`, and points `record` at them.
pub(crate) fn fill(
  record: &mut NativeAnnotation,
  side: &mut Vec<[f32; 2]>,
  points: &[AnnotationPoint],
  smooth: bool,
) {
  let (low, _) = bounds(points);
  let chain = super::path::fitted(points, smooth);
  record.data_offset = u32::try_from(side.len()).unwrap_or(u32::MAX);
  record.data_count = u32::try_from(chain.len()).unwrap_or(u32::MAX);
  side.extend(
    chain
      .iter()
      .map(|point| [point[0] - low.x as f32, point[1] - low.y as f32]),
  );
  let (shares, length) = super::reveal::shares(&chain);
  side.extend(shares);
  record.params[2] = length;
}
