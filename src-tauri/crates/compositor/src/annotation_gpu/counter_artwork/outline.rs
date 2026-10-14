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

/// `channel`, a `cell` of one byte a pixel, blurred about as a Gaussian
/// spread over `softness` pixels would: three passes of a box blur across
/// and three down, each a third of the softness wide either side.
fn blurred(channel: &[u8], (width, height): (usize, usize), softness: f64) -> Vec<u8> {
  let radius = (softness / 3.0).round() as usize;
  if radius == 0 {
    return channel.to_vec();
  }
  let mut values: Vec<f64> = channel.iter().map(|&value| f64::from(value)).collect();
  let span = (2 * radius + 1) as f64;
  let mut line = Vec::new();
  // One pass along every line `count` long, `stride` apart within it and
  // `step` apart from the next line, `lines` of them.
  let mut pass = |values: &mut Vec<f64>, count: usize, stride: usize, step: usize, lines: usize| {
    for start in (0..lines).map(|which| which * step) {
      line.clear();
      line.extend((0..count).map(|at| values[start + at * stride]));
      let at = |index: isize| {
        usize::try_from(index)
          .ok()
          .and_then(|index| line.get(index))
          .copied()
          .unwrap_or(0.0)
      };
      let mut sum: f64 = (-(radius as isize)..=radius as isize).map(at).sum();
      for index in 0..count {
        values[start + index * stride] = sum / span;
        let index = index as isize;
        sum += at(index + radius as isize + 1) - at(index - radius as isize);
      }
    }
  };
  for _ in 0..3 {
    pass(&mut values, width, 1, width, height);
    pass(&mut values, height, width, 1, width);
  }
  values
    .into_iter()
    .map(|value| value.round().clamp(0.0, 255.0) as u8)
    .collect()
}

/// `coverage`, a `cell` of type, as the atlas's BGRA pixels with its outline
/// `width` atlas pixels wide: the type's coverage in the alpha channel, as
/// every cell carries it, and the outline's, the type included, in the red.
/// Where the word casts a shadow, the outline blurred by its `shadow`
/// softness is in the green, which the shader reads set off as the shadow.
pub(super) fn outlined(
  coverage: &[u8],
  cell: (u32, u32),
  width: f64,
  shadow: Option<f64>,
) -> Vec<u8> {
  let size = (cell.0 as usize, cell.1 as usize);
  let distances = squared_distances(coverage, size);
  let outline: Vec<u8> = coverage
    .iter()
    .zip(distances)
    .map(|(&covered, squared)| {
      // A pixel's centre is half a pixel past the edge of the covered one it
      // is nearest, so the outline's edge falls `width` past the type's.
      let reach = (width + 1.0 - squared.sqrt()).clamp(0.0, 1.0);
      ((reach * 255.0).round() as u8).max(covered)
    })
    .collect();
  let shadow = shadow.map(|softness| blurred(&outline, size, softness));
  coverage
    .iter()
    .zip(&outline)
    .enumerate()
    .flat_map(|(index, (&covered, &outline))| {
      let cast = shadow.as_ref().map_or(255, |shadow| shadow[index]);
      [255, cast, outline, covered]
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
    let pixels = outlined(&coverage, (21, 21), 4.0, None);
    let red = |x: usize, y: usize| pixels[(y * 21 + x) * 4 + 2];
    assert_eq!(red(10, 10), 255);
    assert_eq!(red(14, 10), 255);
    assert_eq!(red(10, 15), 0);
    // Round, not square: the corner at the same reach across is outside.
    assert_eq!(red(14, 14), 0);
    // Between full and none at the edge, a pixel in.
    assert!((1..255).contains(&red(13, 13)), "{}", red(13, 13));
  }

  #[test]
  fn casts_its_outline_blurred_beside_it_or_as_it_is_where_hard() {
    let mut coverage = vec![0; 31 * 31];
    coverage[15 * 31 + 15] = 255;
    let green = |pixels: &[u8], x: usize, y: usize| pixels[(y * 31 + x) * 4 + 1];
    let red = |pixels: &[u8], x: usize, y: usize| pixels[(y * 31 + x) * 4 + 2];
    let hard = outlined(&coverage, (31, 31), 4.0, Some(0.0));
    assert!((0..31).all(|x| green(&hard, x, 15) == red(&hard, x, 15)));
    // Soft, it reaches past the outline and is fainter at its middle.
    let soft = outlined(&coverage, (31, 31), 4.0, Some(9.0));
    assert_eq!(red(&soft, 21, 15), 0);
    assert!(green(&soft, 21, 15) > 0);
    assert!(green(&soft, 15, 15) < 255);
  }
}
