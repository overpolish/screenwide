// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::soft_edges;
use rayon::prelude::*;

/// Per-axis neighbour deltas. `gx[y * width + x]` is the largest absolute
/// per-channel difference between pixel `x` and pixel `x - 1` on the same row
/// (column 0 is always 0); `gy` is the same against the row above (row 0 is
/// always 0).
///
/// An element covering columns `L..=R` therefore peaks at `x == L` and at
/// `x == R + 1`, so a component bounding box is naturally half-open `[L, R + 1)`
/// and its reported width matches the element's true width.
pub struct GradientMaps {
  pub gx: Vec<u8>,
  pub gy: Vec<u8>,
  pub width: u32,
  pub height: u32,
  /// Net contrast at settled transition peaks; raw gradients remain unchanged.
  pub soft_edges: Option<(Vec<u8>, Vec<u8>)>,
}

fn channel_delta(a: &[u8], b: &[u8]) -> u8 {
  let red = a[0].abs_diff(b[0]);
  let green = a[1].abs_diff(b[1]);
  let blue = a[2].abs_diff(b[2]);
  red.max(green).max(blue)
}

pub fn compute_gradients(rgba: &[u8], width: u32, height: u32) -> GradientMaps {
  let (w, h) = (width as usize, height as usize);
  let mut gx = vec![0u8; w * h];
  let mut gy = vec![0u8; w * h];
  if w > 0 && h > 0 && rgba.len() >= w * h * 4 {
    gx.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
      let base = y * w * 4;
      for x in 1..w {
        row[x] = channel_delta(&rgba[base + (x - 1) * 4..], &rgba[base + x * 4..]);
      }
    });
    gy.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
      if y == 0 {
        return;
      }
      let (above, base) = ((y - 1) * w * 4, y * w * 4);
      for x in 0..w {
        row[x] = channel_delta(&rgba[above + x * 4..], &rgba[base + x * 4..]);
      }
    });
  }
  let soft_edges = soft_edges::compute(rgba, width, height, &gx, &gy);
  GradientMaps {
    soft_edges,
    gx,
    gy,
    width,
    height,
  }
}

impl GradientMaps {
  pub(crate) fn soft_strength(&self, index: usize, horizontal: bool) -> u8 {
    self.soft_edges.as_ref().map_or(0, |(gx, gy)| {
      let plane = if horizontal { gx } else { gy };
      plane.get(index).copied().unwrap_or(0)
    })
  }

  /// The two directional peaks of a diagonal transition can be one pixel
  /// apart. Combine their contrast only at existing peaks, so flat pixels
  /// between transitions cannot become candidate edge pixels.
  pub(crate) fn has_soft_edge(&self, index: usize, threshold: u8) -> bool {
    let Some((gx, gy)) = &self.soft_edges else {
      return false;
    };
    let (x, y) = (index % self.width as usize, index / self.width as usize);
    let (sx, sy) = (gx[index], gy[index]);
    if sx.max(sy) >= threshold {
      return true;
    }
    if sx == 0 && sy == 0 {
      return false;
    }
    let mut other_x = sx;
    let mut other_y = sy;
    for row in y.saturating_sub(1)..=(y + 1).min(self.height as usize - 1) {
      for col in x.saturating_sub(1)..=(x + 1).min(self.width as usize - 1) {
        let nearby = row * self.width as usize + col;
        if sy > 0 {
          other_x = other_x.max(gx[nearby]);
        }
        if sx > 0 {
          other_y = other_y.max(gy[nearby]);
        }
      }
    }
    u32::from(other_x).pow(2) + u32::from(other_y).pow(2) >= u32::from(threshold).pow(2)
  }
}
