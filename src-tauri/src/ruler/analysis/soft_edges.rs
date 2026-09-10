// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use rayon::prelude::*;

/// Supplement neighbour gradients with the net RGB contrast of a settled,
/// five-pixel transition. Keep only its peak: borrowing contrast must not grow
/// sharp edges or make a flat gap between two objects into an edge.
pub(super) fn compute(
  rgba: &[u8],
  width: u32,
  height: u32,
  gx: &[u8],
  gy: &[u8],
) -> Option<(Vec<u8>, Vec<u8>)> {
  let (w, h) = (width as usize, height as usize);
  if w == 0 || h == 0 || rgba.len() < w * h * 4 {
    return None;
  }
  let plane = |gradients: &[u8], vertical: bool| {
    (0..w * h)
      .into_par_iter()
      .map(|index| {
        let (position, limit, stride) = if vertical {
          (index / w, h, w)
        } else {
          (index % w, w, 1)
        };
        let peak = gradients[index];
        if peak < 2 || position < 3 || position + 3 >= limit {
          return 0;
        }
        // A long slope is not a boundary. Both ends must have settled, and
        // the proposed location must be a local peak in this transition.
        if gradients[index - 3 * stride] >= peak
          || gradients[index + 3 * stride] >= peak
          || (1..=2).any(|offset| {
            gradients[index - offset * stride] > peak || gradients[index + offset * stride] > peak
          })
        {
          return 0;
        }
        let before = (index - 3 * stride) * 4;
        let after = (index + 2 * stride) * 4;
        // Net contrast rejects oscillating noise that an absolute-gradient
        // sum would incorrectly count as a strong transition.
        (0..3)
          .map(|channel| rgba[before + channel].abs_diff(rgba[after + channel]))
          .max()
          .unwrap_or(0)
      })
      .collect()
  };
  Some((plane(gx, false), plane(gy, true)))
}
