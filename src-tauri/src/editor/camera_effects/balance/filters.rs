// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two filters the balance reads a frame through: an erosion, to keep a
//! mask's edge out, and the gradient gray-edge measures.

/// `mask` with each pixel kept only where every pixel within `reach` of it,
/// a square, is kept too. Beyond the frame counts as kept.
pub(super) fn eroded(mask: Vec<bool>, (width, height): (usize, usize), reach: usize) -> Vec<bool> {
  let mut across = vec![false; mask.len()];
  for y in 0..height {
    for x in 0..width {
      let row = y * width;
      across[row + x] =
        (x.saturating_sub(reach)..=(x + reach).min(width - 1)).all(|at| mask[row + at]);
    }
  }
  let mut both = vec![false; mask.len()];
  for y in 0..height {
    for x in 0..width {
      both[y * width + x] =
        (y.saturating_sub(reach)..=(y + reach).min(height - 1)).all(|at| across[at * width + x]);
    }
  }
  both
}

/// How fast each channel of a frame changes at each pixel: lightly blurred,
/// then the Sobel gradient's length.
pub(super) fn edges(linear: &[[f32; 3]], (width, height): (usize, usize)) -> Vec<[f32; 3]> {
  // A Gaussian of about two thirds of a pixel, two pixels of a 960-wide
  // frame at the size the samples are taken.
  const TAPS: [f32; 5] = [0.006_64, 0.194_2, 0.598_2, 0.194_2, 0.006_64];
  let at = |x: isize, y: isize| {
    (y.clamp(0, height as isize - 1) as usize) * width + x.clamp(0, width as isize - 1) as usize
  };
  let blur = |from: &[[f32; 3]], (dx, dy): (isize, isize)| {
    let mut to = vec![[0.0f32; 3]; from.len()];
    for y in 0..height as isize {
      for x in 0..width as isize {
        let sum = &mut to[at(x, y)];
        for (tap, weight) in TAPS.iter().enumerate() {
          let offset = tap as isize - 2;
          for (sum, value) in sum
            .iter_mut()
            .zip(from[at(x + dx * offset, y + dy * offset)])
          {
            *sum += value * weight;
          }
        }
      }
    }
    to
  };
  let blurred = blur(&blur(linear, (1, 0)), (0, 1));
  let mut edges = vec![[0.0f32; 3]; linear.len()];
  for y in 0..height as isize {
    for x in 0..width as isize {
      let value = |dx: isize, dy: isize, channel: usize| blurred[at(x + dx, y + dy)][channel];
      edges[at(x, y)] = [0, 1, 2].map(|c| {
        let gx = value(1, -1, c) + 2.0 * value(1, 0, c) + value(1, 1, c)
          - value(-1, -1, c)
          - 2.0 * value(-1, 0, c)
          - value(-1, 1, c);
        let gy = value(-1, 1, c) + 2.0 * value(0, 1, c) + value(1, 1, c)
          - value(-1, -1, c)
          - 2.0 * value(0, -1, c)
          - value(1, -1, c);
        gx.hypot(gy)
      });
    }
  }
  edges
}
