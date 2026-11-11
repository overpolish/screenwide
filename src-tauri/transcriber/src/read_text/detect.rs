// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding the lines of text: the detection model answers, for every pixel of
//! the picture as it saw it, the chance that it is text. Pixels sure enough
//! join into areas, and each area sure enough on the whole, grown back out
//! to the whole line, is a line's box. The numbers are the model's own
//! (PP-OCRv6 small's `inference.yml`).
//!
//! Screen text runs level, so a box is the area's upright bounds rather than
//! its smallest turned rectangle: the same box for level text, and simpler.

use super::picture::Picture;

/// The shortest side the picture is scaled up to before detection, as the
/// model was made at.
const LEAST_SIDE: usize = 736;
/// The longest side it is scaled down to, to bound the work.
const MOST_SIDE: usize = 2000;
/// How sure a pixel must be to be text.
const PIXEL_SURENESS: f32 = 0.2;
/// How sure an area must be on average to be a line.
const AREA_SURENESS: f32 = 0.45;
/// How far an area is grown back out to the line, as a share of its area
/// over its edge.
const UNCLIP_RATIO: f32 = 1.4;
/// The narrowest an area may be, in the model's pixels.
const LEAST_EXTENT: f32 = 3.0;
/// The most lines kept.
const MOST_AREAS: usize = 3000;
/// How far apart, in pixels, the tops of two lines may be and still be read
/// as one row, left to right.
const SAME_ROW: f32 = 10.0;

const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];

/// A line's box in the picture's pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TextBox {
  pub left: f32,
  pub top: f32,
  pub right: f32,
  pub bottom: f32,
}

/// The size the model sees a `width` by `height` picture at: scaled so its
/// shorter side is at least [`LEAST_SIDE`] and its longer at most
/// [`MOST_SIDE`], each side a multiple of 32.
pub(super) fn model_size(width: usize, height: usize) -> (usize, usize) {
  let mut scale = 1.0_f32;
  let shorter = width.min(height) as f32;
  if shorter < LEAST_SIDE as f32 {
    scale = LEAST_SIDE as f32 / shorter;
  }
  let longer = width.max(height) as f32 * scale;
  if longer > MOST_SIDE as f32 {
    scale = MOST_SIDE as f32 / width.max(height) as f32;
  }
  let side = |length: usize| (((length as f32 * scale) / 32.0).round() as usize * 32).max(32);
  (side(width), side(height))
}

/// The picture as the model takes it: three planes, blue, green and red,
/// each normalised as the model was made with.
pub(super) fn model_input(picture: &Picture<'_>, (width, height): (usize, usize)) -> Vec<f32> {
  let area = width * height;
  let mut planes = vec![0.0; area * 3];
  picture.resample(
    [0, 0, picture.width, picture.height],
    (width, height),
    |x, y, bgr| {
      for channel in 0..3 {
        planes[channel * area + y * width + x] =
          (bgr[channel] / 255.0 - MEAN[channel]) / DEVIATION[channel];
      }
    },
  );
  planes
}

/// The lines in `chances`, the model's answer for a picture it saw at
/// `seen` pixels, as boxes in the `picture` sized original, in reading order.
pub(super) fn boxes(
  chances: &[f32],
  (seen_width, seen_height): (usize, usize),
  (width, height): (usize, usize),
) -> Vec<TextBox> {
  let scale_x = width as f32 / seen_width as f32;
  let scale_y = height as f32 / seen_height as f32;
  let mut found = areas(chances, seen_width, seen_height)
    .into_iter()
    .take(MOST_AREAS)
    .filter_map(|[left, top, right, bottom]| {
      let (extent_x, extent_y) = ((right - left) as f32, (bottom - top) as f32);
      if extent_x.min(extent_y) < LEAST_EXTENT {
        return None;
      }
      let mut sum = 0.0;
      for y in top..=bottom {
        sum += chances[y * seen_width + left..=y * seen_width + right]
          .iter()
          .sum::<f32>();
      }
      let mean = sum / ((right - left + 1) * (bottom - top + 1)) as f32;
      if mean < AREA_SURENESS {
        return None;
      }
      // Grown out by the area over the edge, which turns the shrunken
      // centre the model marks back into the whole line.
      let grow = extent_x * extent_y * UNCLIP_RATIO / (2.0 * (extent_x + extent_y));
      if (extent_x + 2.0 * grow).min(extent_y + 2.0 * grow) < LEAST_EXTENT + 2.0 {
        return None;
      }
      Some(TextBox {
        left: ((left as f32 - grow) * scale_x).clamp(0.0, width as f32),
        top: ((top as f32 - grow) * scale_y).clamp(0.0, height as f32),
        right: ((right as f32 + grow) * scale_x).clamp(0.0, width as f32),
        bottom: ((bottom as f32 + grow) * scale_y).clamp(0.0, height as f32),
      })
    })
    .collect::<Vec<_>>();
  in_reading_order(&mut found);
  found
}

/// The bounds `[left, top, right, bottom]` of each area of touching pixels,
/// corners included, sure enough to be text, in the order they are met.
fn areas(chances: &[f32], width: usize, height: usize) -> Vec<[usize; 4]> {
  let mut seen = chances
    .iter()
    .map(|&chance| chance <= PIXEL_SURENESS)
    .collect::<Vec<_>>();
  let mut found = Vec::new();
  let mut stack = Vec::new();
  for start in 0..seen.len() {
    if seen[start] {
      continue;
    }
    seen[start] = true;
    stack.push(start);
    let mut bounds = [usize::MAX, usize::MAX, 0, 0];
    while let Some(at) = stack.pop() {
      let (x, y) = (at % width, at / width);
      bounds = [
        bounds[0].min(x),
        bounds[1].min(y),
        bounds[2].max(x),
        bounds[3].max(y),
      ];
      for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
        for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
          let next = ny * width + nx;
          if !seen[next] {
            seen[next] = true;
            stack.push(next);
          }
        }
      }
    }
    found.push(bounds);
  }
  found
}

/// Top to bottom, and left to right along a row: PaddleOCR's order.
fn in_reading_order(boxes: &mut [TextBox]) {
  boxes.sort_by(|a, b| a.top.total_cmp(&b.top).then(a.left.total_cmp(&b.left)));
  for end in 1..boxes.len() {
    let mut at = end;
    while at > 0
      && (boxes[at].top - boxes[at - 1].top).abs() < SAME_ROW
      && boxes[at].left < boxes[at - 1].left
    {
      boxes.swap(at, at - 1);
      at -= 1;
    }
  }
}
