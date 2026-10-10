// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An object's mask drawn at the matte's size with a clean edge. Cutie's
//! mask is a quarter of its frame's size, about nine pixels of a 1080p frame
//! to each of its own: stretched as it is, every one of them shows on a
//! straight edge. So, as Cutie draws its own masks, the odds are stretched
//! rather than the probabilities, which leaves a smooth outline.
//!
//! A kept object's outline is then moved onto its own edge by a guided
//! filter (He et al.) on the camera frame, a phone's hard edge staying hard.
//! A removed object is taken away whole and a little past its outline
//! instead: the filter puts a bright rim, such as a phone's, with the room,
//! and a rim left behind shows as a line around where the object was.

use super::super::track::{MASK_HEIGHT, MASK_WIDTH};

/// How far the guided filter looks, and how much a change of brightness
/// must stand out from the frame's grain to count as an edge, for a matte
/// 540 lines high.
const RADIUS: f32 = 6.0;
const GRAIN: f64 = 1e-3;
/// How far a removed object's outline is pushed out, in its odds, then
/// where between unsure and sure it is taken away in full, and how far past
/// that it is grown, in pixels of a matte 540 lines high.
const REMOVED_REACH: f32 = 0.5;
const REMOVED_FROM: f32 = 0.3;
const REMOVED_FULLY: f32 = 0.7;
const REMOVED_GROWTH: f32 = 4.0;
/// Mask pixels less likely than this are not the object.
const NOTHING: f32 = 0.02;

/// For each pixel of the matte along one side, the two mask pixels either
/// side of it and the share of the second, as a bilinear stretch takes them.
type Taps = Vec<(usize, usize, f32)>;

pub(super) struct Edge {
  width: usize,
  across: Taps,
  down: Taps,
  radius: usize,
  growth: usize,
  /// The area of the frame the object reaches, and its probability there.
  area: (usize, usize, usize, usize),
  object: Vec<f32>,
}

impl Edge {
  pub(super) fn new(width: usize, height: usize) -> Self {
    let taps = |pixels: usize, mask: usize| -> Taps {
      let last = (mask - 1) as f32;
      (0..pixels)
        .map(|pixel| {
          let at = ((pixel as f32 + 0.5) * mask as f32 / pixels as f32 - 0.5).clamp(0.0, last);
          let first = at.floor() as usize;
          (first, (first + 1).min(mask - 1), at.fract())
        })
        .collect()
    };
    Self {
      width,
      across: taps(width, MASK_WIDTH),
      down: taps(height, MASK_HEIGHT),
      radius: (RADIUS * height as f32 / 540.0).round().max(2.0) as usize,
      growth: (REMOVED_GROWTH * height as f32 / 540.0).round().max(1.0) as usize,
      area: (0, 0, 0, 0),
      object: Vec::new(),
    }
  }

  /// `mask`, an object's probability 0 to 255 at the mask's size, added to
  /// `alpha` or, where not `keep`, taken from it. `guide` is the camera
  /// frame's brightness, 0 to 1, at the matte's size.
  pub(super) fn apply(&mut self, alpha: &mut [u8], mask: &[u8], guide: &[f32], keep: bool) {
    let Some(cells) = extent(mask) else {
      return;
    };
    if keep {
      self.draw(mask, cells, 0.0);
      self.guide(guide);
    } else {
      self.draw(mask, cells, REMOVED_REACH);
      self.firm();
    }
    let (left, top, right, bottom) = self.area;
    let span = right - left;
    for y in top..bottom {
      let row = &mut alpha[y * self.width..(y + 1) * self.width];
      let object = &self.object[(y - top) * span..][..span];
      for (x, &value) in (left..right).zip(object) {
        let current = f32::from(row[x]) / 255.0;
        // Kept, the object is united with what is there: where its soft edge
        // meets the person's, as a held phone meets the hand, the larger of
        // two halves would leave a dark seam between them.
        let next = if keep {
          current + value - current * value
        } else {
          current.min(1.0 - value)
        };
        row[x] = (next.clamp(0.0, 1.0) * 255.0).round() as u8;
      }
    }
  }

  /// The object's probability over the frame's pixels its cells reach, and
  /// the guided filter's reach around them: its odds stretched, then made
  /// a probability again, as Cutie's softmax over the object and the room.
  fn draw(
    &mut self,
    mask: &[u8],
    (left, top, right, bottom): (usize, usize, usize, usize),
    reach: f32,
  ) {
    let logits = mask
      .iter()
      .map(|&byte| {
        let p = (f32::from(byte) / 255.0).clamp(1.0 / 255.0, 254.0 / 255.0);
        (p / (1.0 - p)).ln()
      })
      .collect::<Vec<_>>();
    let span = |taps: &Taps, low: usize, high: usize, pixels: usize| {
      let from = taps
        .partition_point(|tap| tap.1 < low)
        .saturating_sub(self.radius);
      let to = (taps.partition_point(|tap| tap.0 <= high) + self.radius).min(pixels);
      (from, to)
    };
    let (x0, x1) = span(&self.across, left, right, self.across.len());
    let (y0, y1) = span(&self.down, top, bottom, self.down.len());
    self.area = (x0, y0, x1, y1);
    self.object.clear();
    for y in y0..y1 {
      let (r0, r1, fy) = self.down[y];
      for x in x0..x1 {
        let (c0, c1, fx) = self.across[x];
        let at = |row: usize, column: usize| logits[row * MASK_WIDTH + column];
        let upper = at(r0, c0) * (1.0 - fx) + at(r0, c1) * fx;
        let lower = at(r1, c0) * (1.0 - fx) + at(r1, c1) * fx;
        let logit = upper * (1.0 - fy) + lower * fy + reach;
        self.object.push(1.0 / (1.0 + (-2.0 * logit).exp()));
      }
    }
  }

  /// The object's probability moved onto the edges of `guide` by a guided
  /// filter over its area.
  fn guide(&mut self, guide: &[f32]) {
    let (left, top, right, bottom) = self.area;
    let (w, h) = (right - left, bottom - top);
    let image = (top..bottom)
      .flat_map(|y| {
        guide[y * self.width + left..y * self.width + right]
          .iter()
          .copied()
      })
      .collect::<Vec<_>>();
    let mean = |values: &[f64]| boxed(values, (w, h), self.radius);
    let as_f64 = |values: &[f32]| {
      values
        .iter()
        .map(|&value| f64::from(value))
        .collect::<Vec<_>>()
    };
    let (i, p) = (as_f64(&image), as_f64(&self.object));
    let products = i.iter().zip(&p).map(|(a, b)| a * b).collect::<Vec<_>>();
    let squares = i.iter().map(|a| a * a).collect::<Vec<_>>();
    let (mean_i, mean_p) = (mean(&i), mean(&p));
    let (mean_ip, mean_ii) = (mean(&products), mean(&squares));
    let mut a = vec![0.0; w * h];
    let mut b = vec![0.0; w * h];
    for index in 0..w * h {
      let covariance = mean_ip[index] - mean_i[index] * mean_p[index];
      let variance = mean_ii[index] - mean_i[index] * mean_i[index];
      a[index] = covariance / (variance + GRAIN);
      b[index] = mean_p[index] - a[index] * mean_i[index];
    }
    let (mean_a, mean_b) = (mean(&a), mean(&b));
    for (index, value) in self.object.iter_mut().enumerate() {
      *value = (mean_a[index] * i[index] + mean_b[index]).clamp(0.0, 1.0) as f32;
    }
  }

  /// A removed object made sure where the model is fairly sure, grown past
  /// its outline, and its new edge softened by a pixel.
  fn firm(&mut self) {
    let (left, top, right, bottom) = self.area;
    let (w, h) = (right - left, bottom - top);
    for value in &mut self.object {
      *value = ((*value - REMOVED_FROM) / (REMOVED_FULLY - REMOVED_FROM)).clamp(0.0, 1.0);
    }
    // The most of each square `growth` either side, along the rows and
    // then down the columns.
    let reach = self.growth;
    let mut across = vec![0.0_f32; w * h];
    for y in 0..h {
      let row = &self.object[y * w..(y + 1) * w];
      for x in 0..w {
        let (from, to) = (x.saturating_sub(reach), (x + reach + 1).min(w));
        across[y * w + x] = row[from..to].iter().copied().fold(0.0, f32::max);
      }
    }
    let mut grown = vec![0.0_f64; w * h];
    for x in 0..w {
      for y in 0..h {
        let (from, to) = (y.saturating_sub(reach), (y + reach + 1).min(h));
        grown[y * w + x] = f64::from(
          (from..to)
            .map(|row| across[row * w + x])
            .fold(0.0, f32::max),
        );
      }
    }
    for (value, soft) in self.object.iter_mut().zip(boxed(&grown, (w, h), 1)) {
      *value = soft as f32;
    }
  }
}

/// The mean of `values`, `size` across and down, over the square `radius`
/// either side of each, cut short at the edges.
fn boxed(values: &[f64], (w, h): (usize, usize), radius: usize) -> Vec<f64> {
  let mut sums = vec![0.0_f64; (w + 1) * (h + 1)];
  for y in 0..h {
    let mut row = 0.0;
    for x in 0..w {
      row += values[y * w + x];
      sums[(y + 1) * (w + 1) + x + 1] = sums[y * (w + 1) + x + 1] + row;
    }
  }
  let mut means = vec![0.0; w * h];
  for y in 0..h {
    let (y0, y1) = (y.saturating_sub(radius), (y + radius + 1).min(h));
    for x in 0..w {
      let (x0, x1) = (x.saturating_sub(radius), (x + radius + 1).min(w));
      let total = sums[y1 * (w + 1) + x1] - sums[y0 * (w + 1) + x1] - sums[y1 * (w + 1) + x0]
        + sums[y0 * (w + 1) + x0];
      means[y * w + x] = total / ((y1 - y0) * (x1 - x0)) as f64;
    }
  }
  means
}

/// The first and last column and row of `mask` the object reaches, or
/// nothing where it is not in the frame.
fn extent(mask: &[u8]) -> Option<(usize, usize, usize, usize)> {
  let least = (NOTHING * 255.0) as u8;
  let mut found: Option<(usize, usize, usize, usize)> = None;
  for (index, &value) in mask.iter().enumerate() {
    if value < least {
      continue;
    }
    let (x, y) = (index % MASK_WIDTH, index / MASK_WIDTH);
    found = Some(match found {
      None => (x, y, x, y),
      Some((left, top, right, bottom)) => (left.min(x), top.min(y), right.max(x), bottom.max(y)),
    });
  }
  found
}
