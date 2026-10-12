// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::luma::{Plane, Pyramid};

/// Half the flow window's side: a 15 pixel square.
const HALF: usize = 7;
const WINDOW: usize = 2 * HALF + 1;
/// The window plus the one pixel border its central differences read.
const PATCH: usize = WINDOW + 2;
const MAX_ITERATIONS: usize = 10;
/// A step this small, in pixels, has converged.
const CONVERGED: f32 = 0.02;
/// The weakest window structure followed at the finest level, as the smaller
/// eigenvalue averaged per pixel: about a two step gradient in each
/// direction. Weaker, a window on a line or on flat colour slides.
const MIN_STRUCTURE: f32 = 4.0;

/// `size` by `size` bilinear samples of `plane` whose top-left sample is at
/// `(x, y)`. Every sample in a window shares one fractional offset, so the
/// four weights are worked out once.
fn sample(plane: &Plane, x: f32, y: f32, size: usize, out: &mut [f32]) {
  let (fx, fy) = (x.floor(), y.floor());
  let (ax, ay) = (x - fx, y - fy);
  let (w00, w10, w01, w11) = (
    (1.0 - ax) * (1.0 - ay),
    ax * (1.0 - ay),
    (1.0 - ax) * ay,
    ax * ay,
  );
  let (ix, iy) = (fx as isize, fy as isize);
  let inside =
    ix >= 0 && iy >= 0 && (ix as usize) + size < plane.width && (iy as usize) + size < plane.height;
  if inside {
    let (ix, iy) = (ix as usize, iy as usize);
    for row in 0..size {
      let top = &plane.data[(iy + row) * plane.width + ix..][..size + 1];
      let bottom = &plane.data[(iy + row + 1) * plane.width + ix..][..size + 1];
      let target = &mut out[row * size..][..size];
      for column in 0..size {
        target[column] = w00 * top[column]
          + w10 * top[column + 1]
          + w01 * bottom[column]
          + w11 * bottom[column + 1];
      }
    }
  } else {
    for row in 0..size {
      let y = iy + row as isize;
      for column in 0..size {
        let x = ix + column as isize;
        out[row * size + column] = w00 * plane.at_clamped(x, y)
          + w10 * plane.at_clamped(x + 1, y)
          + w01 * plane.at_clamped(x, y + 1)
          + w11 * plane.at_clamped(x + 1, y + 1);
      }
    }
  }
}

/// Where `point` in `prev` has gone in `next`, starting the search at
/// `guess`. `None` where the window has too little structure to follow or
/// the answer leaves the frame.
pub(crate) fn track(
  prev: &Pyramid,
  next: &Pyramid,
  point: [f32; 2],
  guess: [f32; 2],
) -> Option<[f32; 2]> {
  let mut patch = [0.0_f32; PATCH * PATCH];
  let mut moved = [0.0_f32; WINDOW * WINDOW];
  let mut template = [0.0_f32; WINDOW * WINDOW];
  let mut gradient_x = [0.0_f32; WINDOW * WINDOW];
  let mut gradient_y = [0.0_f32; WINDOW * WINDOW];

  let top = prev.levels.len() - 1;
  let top_scale = (1_usize << top) as f32;
  let mut g = [
    (guess[0] - point[0]) / top_scale,
    (guess[1] - point[1]) / top_scale,
  ];
  for level in (0..=top).rev() {
    let scale = (1_usize << level) as f32;
    let (prev_plane, next_plane) = (&prev.levels[level], &next.levels[level]);
    let (px, py) = (point[0] / scale, point[1] / scale);
    sample(
      prev_plane,
      px - (HALF + 1) as f32,
      py - (HALF + 1) as f32,
      PATCH,
      &mut patch,
    );
    let (mut gxx, mut gyy, mut gxy) = (0.0_f32, 0.0, 0.0);
    for row in 0..WINDOW {
      for column in 0..WINDOW {
        let centre = (row + 1) * PATCH + column + 1;
        let dx = 0.5 * (patch[centre + 1] - patch[centre - 1]);
        let dy = 0.5 * (patch[centre + PATCH] - patch[centre - PATCH]);
        let index = row * WINDOW + column;
        template[index] = patch[centre];
        gradient_x[index] = dx;
        gradient_y[index] = dy;
        gxx += dx * dx;
        gyy += dy * dy;
        gxy += dx * dy;
      }
    }
    let determinant = gxx * gyy - gxy * gxy;
    let min_eigen = (0.5 * (gxx + gyy) - (0.25 * (gxx - gyy) * (gxx - gyy) + gxy * gxy).sqrt())
      / (WINDOW * WINDOW) as f32;
    if determinant.abs() <= f32::EPSILON || (level == 0 && min_eigen < MIN_STRUCTURE) {
      if level == 0 {
        return None;
      }
      g = [2.0 * g[0], 2.0 * g[1]];
      continue;
    }
    let mut v = [0.0_f32; 2];
    for _ in 0..MAX_ITERATIONS {
      sample(
        next_plane,
        px + g[0] + v[0] - HALF as f32,
        py + g[1] + v[1] - HALF as f32,
        WINDOW,
        &mut moved,
      );
      let (mut bx, mut by) = (0.0_f32, 0.0);
      for index in 0..WINDOW * WINDOW {
        let difference = template[index] - moved[index];
        bx += difference * gradient_x[index];
        by += difference * gradient_y[index];
      }
      let step = [
        (gyy * bx - gxy * by) / determinant,
        (gxx * by - gxy * bx) / determinant,
      ];
      v[0] += step[0];
      v[1] += step[1];
      if step[0] * step[0] + step[1] * step[1] < CONVERGED * CONVERGED {
        break;
      }
    }
    g = if level == 0 {
      [g[0] + v[0], g[1] + v[1]]
    } else {
      [2.0 * (g[0] + v[0]), 2.0 * (g[1] + v[1])]
    };
  }
  let found = [point[0] + g[0], point[1] + g[1]];
  let inside =
    found[0] >= 0.0 && found[1] >= 0.0 && found[0] < next.width() && found[1] < next.height();
  (inside && found[0].is_finite() && found[1].is_finite()).then_some(found)
}
