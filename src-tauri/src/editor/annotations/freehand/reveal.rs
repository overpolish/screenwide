// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How much of a stroke shows while it draws itself in and out.
//!
//! A timed annotation reveals the stretch of its path from `low` to `high`,
//! both shares of its length, and a stroke's path is its fitted chain. The
//! reveal moves every frame while the chain is packed once - a video export
//! packs it for the whole clip - so the chain is not cut here. Instead each
//! curve carries where along the stroke it starts and ends, as shares of the
//! stroke's length, and the shader draws of each curve only the part the
//! window covers. Lengths are measured along each curve rather than taken
//! from its parameter, which runs unevenly round a bend; within one of a
//! chain's short curves the difference is too small to see.

use crate::editor::annotations::geometry::bezier;

/// How many straight steps a curve's length is measured in.
const STEPS: usize = 8;

/// Where along the stroke each curve of `chain` starts and ends, as shares
/// of the whole chain's length, and that length. A chain of no length is
/// shown whole.
pub(crate) fn shares(chain: &[[f32; 2]]) -> (Vec<[f32; 2]>, f32) {
  let lengths: Vec<f32> = (0..chain.len().saturating_sub(1) / 2)
    .map(|curve| length(chain[curve * 2], chain[curve * 2 + 1], chain[curve * 2 + 2]))
    .collect();
  let total: f32 = lengths.iter().sum();
  if total <= 0.0 {
    return (vec![[0.0, 1.0]; lengths.len()], 0.0);
  }
  let mut start = 0.0;
  let shares = lengths
    .iter()
    .map(|length| {
      let end = start + length;
      let share = [start / total, end / total];
      start = end;
      share
    })
    .collect();
  (shares, total)
}

/// The length of the quadratic curve `a, b, c`.
fn length(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
  let mut total = 0.0;
  let mut previous = a;
  for step in 1..=STEPS {
    let point = bezier(a, b, c, step as f32 / STEPS as f32);
    total += (point[0] - previous[0]).hypot(point[1] - previous[1]);
    previous = point;
  }
  total
}
