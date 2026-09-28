// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A highlight's retained draw record.
//!
//! `p0` and `p1` are the source's origin and its pixel `(1, 1)`, which every
//! placement carries into the pixels the highlight is drawn in, and `p2` its
//! tone, the page's brightness and its ink's. Its bands follow in the side
//! buffer's points, `data_count` of them from `data_offset`: each band's
//! top-left corner and then its bottom-right, in source pixels. `flags`
//! carries `HAND_DRAWN`, and `params` the seed's low and high halves, each
//! exact in a float, and the sweep `exposure::highlight_travel` reads.

use super::model::HighlightBand;
use crate::editor::annotations::flags::HAND_DRAWN;
use crate::editor::annotations::native::NativeAnnotation;

/// Writes the bands into `points` and points `record` at them.
pub(crate) fn fill(
  record: &mut NativeAnnotation,
  points: &mut Vec<[f32; 2]>,
  bands: &[HighlightBand],
  seed: u32,
  hand_drawn: bool,
) {
  record.data_offset = u32::try_from(points.len()).unwrap_or(u32::MAX);
  record.data_count = u32::try_from(bands.len() * 2).unwrap_or(u32::MAX);
  for band in bands {
    points.push([band.left as f32, band.top as f32]);
    points.push([band.right as f32, band.bottom as f32]);
  }
  record.flags = if hand_drawn { HAND_DRAWN } else { 0 };
  record.params = [(seed & 0xffff) as f32, (seed >> 16) as f32, sweep(bands)];
}

/// How far the fastest line end runs over the whole reveal, in source pixels:
/// each line draws itself in over its own share of the reveal, so the longest
/// line covers the most ground in the least of it.
fn sweep(bands: &[HighlightBand]) -> f32 {
  let longest = bands
    .iter()
    .map(|band| (band.right - band.left).max(0.0))
    .fold(0.0, f64::max);
  (longest / line_share(bands.len())) as f32
}

/// The share of the reveal each of `lines` lines draws itself in over. The
/// twin of `share` in `composite_highlights`, in `annotation_highlight.hlsl`
/// and `gpu_compositor_macos_shader_source_annotation_highlight.h`.
fn line_share(lines: usize) -> f64 {
  (2.0 / (lines as f64 + 1.0)).clamp(0.35, 1.0)
}
