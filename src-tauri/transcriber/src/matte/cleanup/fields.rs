// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the cleanup measures a frame with: the areas a mask falls into, how
//! far each pixel is from the edge of a mask, and blurs.

use std::ops::Range;

/// One area: how many pixels, and where they sit in the order `label` left.
pub(super) struct Area {
  pub(super) size: usize,
  pub(super) pixels: Range<usize>,
}

/// The areas of pixels where `mask` is `value`, touching side by side or
/// above and below. Their pixels are left in `order`, each area's together.
pub(super) fn label(
  mask: &[bool],
  value: bool,
  (width, height): (usize, usize),
  seen: &mut [bool],
  order: &mut Vec<usize>,
) -> Vec<Area> {
  seen.fill(false);
  order.clear();
  let mut areas = Vec::new();
  for start in 0..mask.len() {
    if mask[start] != value || seen[start] {
      continue;
    }
    let first = order.len();
    seen[start] = true;
    order.push(start);
    let mut next = first;
    while next < order.len() {
      let index = order[next];
      next += 1;
      let (x, y) = (index % width, index / width);
      let mut visit = |neighbour: usize| {
        if mask[neighbour] == value && !seen[neighbour] {
          seen[neighbour] = true;
          order.push(neighbour);
        }
      };
      if x > 0 {
        visit(index - 1);
      }
      if x + 1 < width {
        visit(index + 1);
      }
      if y > 0 {
        visit(index - width);
      }
      if y + 1 < height {
        visit(index + width);
      }
    }
    areas.push(Area {
      size: order.len() - first,
      pixels: first..order.len(),
    });
  }
  areas
}

/// How far, in pixels, each pixel where `mask` is `value` is from the
/// nearest pixel where it is not; zero elsewhere. Exact, as the straight
/// line runs; a mask that is `value` everywhere answers a distance beyond
/// the frame.
pub(super) fn distance(
  mask: &[bool],
  value: bool,
  (width, height): (usize, usize),
  into: &mut [f32],
  scratch: &mut Vec<f32>,
) {
  // Beyond any distance in the frame, and small enough that sums with it
  // keep their whole pixels in an f32.
  let far = ((width + height) * (width + height)) as f32;
  for (out, &inside) in into.iter_mut().zip(mask) {
    *out = if inside == value { far } else { 0.0 };
  }
  scratch.resize(width.max(height) * 3 + 1, 0.0);
  // Squared distances along each column, then along each row.
  for x in 0..width {
    squared_1d(into, (x, width, height), scratch);
  }
  for y in 0..height {
    squared_1d(into, (y * width, 1, width), scratch);
  }
  for value in into.iter_mut() {
    *value = value.sqrt();
  }
}

/// Felzenszwalb and Huttenlocher's lower envelope of parabolas: the squared
/// distances along `line`, the `count` values from `start` a `stride` apart,
/// in place.
fn squared_1d(
  line: &mut [f32],
  (start, stride, count): (usize, usize, usize),
  scratch: &mut [f32],
) {
  let (values, rest) = scratch.split_at_mut(count);
  let (vertices, rest) = rest.split_at_mut(count);
  let bounds = &mut rest[..=count];
  for (index, value) in values.iter_mut().enumerate() {
    *value = line[start + index * stride];
  }
  let crossing = |q: usize, v: usize, values: &[f32]| {
    let (q, v_at) = (q as f32, v as f32);
    ((values[q as usize] + q * q) - (values[v] + v_at * v_at)) / (2.0 * (q - v_at))
  };
  let mut k = 0;
  vertices[0] = 0.0;
  bounds[0] = f32::NEG_INFINITY;
  bounds[1] = f32::INFINITY;
  for q in 1..count {
    let mut s = crossing(q, vertices[k] as usize, values);
    // The first bound is minus infinity, so this stops at the first vertex.
    while s <= bounds[k] {
      k -= 1;
      s = crossing(q, vertices[k] as usize, values);
    }
    k += 1;
    vertices[k] = q as f32;
    bounds[k] = s;
    bounds[k + 1] = f32::INFINITY;
  }
  let mut k = 0;
  for q in 0..count {
    let at = q as f32;
    while bounds[k + 1] < at {
      k += 1;
    }
    let v = vertices[k];
    line[start + q * stride] = (at - v) * (at - v) + values[v as usize];
  }
}

/// `field` blurred in place to about a Gaussian of deviation `sigma` pixels,
/// as three box blurs one after another, each pixel's cost the same however
/// wide the blur.
pub(super) fn blur(
  field: &mut [f32],
  scratch: &mut Vec<f32>,
  (width, height): (usize, usize),
  sigma: f32,
) {
  if sigma < 0.3 {
    return;
  }
  // The box width three passes of which spread as far as the Gaussian.
  let radius = (((12.0 * sigma * sigma / 3.0 + 1.0).sqrt() - 1.0) / 2.0)
    .round()
    .max(1.0) as usize;
  scratch.resize(width.max(height), 0.0);
  for _ in 0..3 {
    for y in 0..height {
      box_1d(field, (y * width, 1, width), radius, scratch);
    }
    for x in 0..width {
      box_1d(field, (x, width, height), radius, scratch);
    }
  }
}

/// The mean of each value and those within `radius` along `line`, the edge
/// values standing in past its ends.
fn box_1d(
  line: &mut [f32],
  (start, stride, count): (usize, usize, usize),
  radius: usize,
  held: &mut [f32],
) {
  for (index, value) in held.iter_mut().take(count).enumerate() {
    *value = line[start + index * stride];
  }
  let at = |index: isize| held[index.clamp(0, count as isize - 1) as usize];
  let span = (2 * radius + 1) as f32;
  let r = radius as isize;
  let mut sum = (-r..=r).map(at).sum::<f32>();
  for index in 0..count as isize {
    line[start + index as usize * stride] = sum / span;
    sum += at(index + r + 1) - at(index - r);
  }
}

/// `mask` grown by `reach` pixels in the four directions, or shrunk where
/// `grow` is false, as many single steps.
pub(super) fn step(
  mask: &mut [bool],
  scratch: &mut Vec<bool>,
  (width, height): (usize, usize),
  reach: usize,
  grow: bool,
) {
  scratch.resize(mask.len(), false);
  for _ in 0..reach {
    scratch.copy_from_slice(mask);
    for (index, pixel) in mask.iter_mut().enumerate() {
      let (x, y) = (index % width, index / width);
      let neighbours = [
        (x > 0).then(|| index - 1),
        (x + 1 < width).then(|| index + 1),
        (y > 0).then(|| index - width),
        (y + 1 < height).then(|| index + width),
      ];
      let changes = neighbours
        .into_iter()
        .map(|neighbour| neighbour.map_or(!grow, |neighbour| scratch[neighbour]))
        .any(|other| other == grow);
      if changes {
        *pixel = grow;
      }
    }
  }
}

#[cfg(test)]
mod tests;
