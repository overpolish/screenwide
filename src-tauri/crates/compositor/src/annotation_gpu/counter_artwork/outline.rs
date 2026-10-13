// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption word's outline, drawn into its atlas cell beside its type so the
//! shader reads it with the same filtered taps, and its edge is as smooth as
//! the type's. The outline is everything within its width of the type,
//! measured as a true distance, so its corners come out round however wide.

/// Farther than any cell is across, squared: what a pixel with no type in its
/// row or column starts as.
const FAR: f64 = 1e12;

/// The squared distance from each of `costs`' places to the nearest one,
/// adding that one's own cost: the lower envelope of the parabolas rooted at
/// each place (Felzenszwalb and Huttenlocher). `roots` and `bounds` are
/// scratch, as long as `costs` and one longer.
fn distances_1d(costs: &[f64], out: &mut [f64], roots: &mut [usize], bounds: &mut [f64]) {
  let count = costs.len();
  if count == 0 {
    return;
  }
  let meet = |q: usize, p: usize| {
    let (qf, pf) = (q as f64, p as f64);
    ((costs[q] + qf * qf) - (costs[p] + pf * pf)) / (2.0 * (qf - pf))
  };
  let mut k = 0;
  roots[0] = 0;
  bounds[0] = f64::NEG_INFINITY;
  bounds[1] = f64::INFINITY;
  for q in 1..count {
    let mut s = meet(q, roots[k]);
    while s <= bounds[k] {
      k -= 1;
      s = meet(q, roots[k]);
    }
    k += 1;
    roots[k] = q;
    bounds[k] = s;
    bounds[k + 1] = f64::INFINITY;
  }
  k = 0;
  for (q, slot) in out.iter_mut().enumerate() {
    while bounds[k + 1] < q as f64 {
      k += 1;
    }
    let along = q as f64 - roots[k] as f64;
    *slot = along * along + costs[roots[k]];
  }
}

/// The squared distance from every pixel of a `cell` to the nearest one
/// mostly covered by type, row by row after column by column.
fn squared_distances(coverage: &[u8], (width, height): (usize, usize)) -> Vec<f64> {
  let mut field: Vec<f64> = coverage
    .iter()
    .map(|&covered| if covered >= 128 { 0.0 } else { FAR })
    .collect();
  let longest = width.max(height);
  let (mut line, mut out) = (vec![0.0; longest], vec![0.0; longest]);
  let (mut roots, mut bounds) = (vec![0; longest], vec![0.0; longest + 1]);
  for x in 0..width {
    for y in 0..height {
      line[y] = field[y * width + x];
    }
    distances_1d(&line[..height], &mut out[..height], &mut roots, &mut bounds);
    for y in 0..height {
      field[y * width + x] = out[y];
    }
  }
  for y in 0..height {
    let row = &mut field[y * width..(y + 1) * width];
    line[..width].copy_from_slice(row);
    distances_1d(&line[..width], &mut out[..width], &mut roots, &mut bounds);
    row.copy_from_slice(&out[..width]);
  }
  field
}

/// `coverage`, a `cell` of type, as the atlas's BGRA pixels with its outline
/// `width` atlas pixels wide: the type's coverage in the alpha channel, as
/// every cell carries it, and the outline's, the type included, in the red.
pub(super) fn outlined(coverage: &[u8], cell: (u32, u32), width: f64) -> Vec<u8> {
  let size = (cell.0 as usize, cell.1 as usize);
  let distances = squared_distances(coverage, size);
  coverage
    .iter()
    .zip(distances)
    .flat_map(|(&covered, squared)| {
      // A pixel's centre is half a pixel past the edge of the covered one it
      // is nearest, so the outline's edge falls `width` past the type's.
      let reach = (width + 1.0 - squared.sqrt()).clamp(0.0, 1.0);
      let outline = (reach * 255.0).round() as u8;
      [255, 255, outline.max(covered), covered]
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reaches_its_width_round_a_point_with_a_soft_edge() {
    // One covered pixel in the middle of a 21 by 21 cell.
    let mut coverage = vec![0; 21 * 21];
    coverage[10 * 21 + 10] = 255;
    let pixels = outlined(&coverage, (21, 21), 4.0);
    let red = |x: usize, y: usize| pixels[(y * 21 + x) * 4 + 2];
    assert_eq!(red(10, 10), 255);
    assert_eq!(red(14, 10), 255);
    assert_eq!(red(10, 15), 0);
    // Round, not square: the corner at the same reach across is outside.
    assert_eq!(red(14, 14), 0);
    // Between full and none at the edge, a pixel in.
    assert!((1..255).contains(&red(13, 13)), "{}", red(13, 13));
  }
}
