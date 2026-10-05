// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's shadow, as the image atlas holds it: its picture's
//! silhouette, blurred. The shader lays it under the picture, lowered and
//! darkening what is there, the way a magnifier's loupe casts its own.
//!
//! A shadow is all soft edge, so it is drawn small - the shader's filtered
//! read spreads it over any size - and blurred once here rather than taken
//! in many reads for every pixel it covers.

use super::{rasterize, StoredImage};

/// The longer side a shadow's silhouette is drawn at, in atlas pixels,
/// before its margin.
pub(crate) const SHADOW_SIDE: u32 = 64;

/// How far a shadow spreads, as a share of half the picture's shorter side:
/// the magnifier's own. The twin of `annotation_image_spread` in
/// `annotation_image.wgsl`.
const SHADOW_SPREAD: f64 = 0.11;

/// Half the shorter side of the picture a cell of `cell` pixels holds,
/// inside its pixel of margin.
fn half_shorter(cell: (u32, u32)) -> f64 {
  f64::from(cell.0.min(cell.1).saturating_sub(2)) / 2.0
}

/// How far the blur reaches past the picture's cell on every side, in
/// pixels: three times its spread, past which it is nothing.
pub(crate) fn shadow_margin(cell: (u32, u32)) -> u32 {
  (3.0 * SHADOW_SPREAD * half_shorter(cell)).ceil() as u32
}

/// The silhouette of `picture` drawn into a cell of `cell` pixels, as
/// [`rasterize`] draws it, blurred and grown by [`shadow_margin`] on every
/// side: BGRA rows, top row first, black with the shadow's strength in the
/// alpha.
pub(crate) fn rasterize_shadow(picture: &StoredImage, cell: (u32, u32)) -> Option<Vec<u8>> {
  let silhouette = rasterize(picture, cell)?;
  let margin = shadow_margin(cell) as usize;
  let (width, height) = (cell.0 as usize + 2 * margin, cell.1 as usize + 2 * margin);
  let mut alpha = vec![0.0_f32; width * height];
  for (row, line) in silhouette
    .as_chunks::<4>()
    .0
    .chunks(cell.0 as usize)
    .enumerate()
  {
    let start = (row + margin) * width + margin;
    for (out, pixel) in alpha[start..start + line.len()].iter_mut().zip(line) {
      *out = f32::from(pixel[3]) / 255.0;
    }
  }
  let spread = SHADOW_SPREAD * half_shorter(cell);
  if spread >= 0.5 {
    let weights: Vec<f32> = (-(margin as i64)..=margin as i64)
      .map(|offset| (-0.5 * (offset as f64 / spread).powi(2)).exp() as f32)
      .collect();
    let total: f32 = weights.iter().sum();
    let weights: Vec<f32> = weights.iter().map(|weight| weight / total).collect();
    alpha = blurred(&alpha, (width, height), &weights, (1, width));
    alpha = blurred(&alpha, (width, height), &weights, (width, 1));
  }
  Some(
    alpha
      .into_iter()
      .flat_map(|share| [0, 0, 0, (share.clamp(0.0, 1.0) * 255.0).round() as u8])
      .collect(),
  )
}

/// `plane` blurred along one axis by `weights`, centred on their middle.
/// `step` is how far apart neighbours along that axis are and how far apart
/// the lines across it are; the plane's edges are clear.
fn blurred(
  plane: &[f32],
  (width, height): (usize, usize),
  weights: &[f32],
  step: (usize, usize),
) -> Vec<f32> {
  let reach = weights.len() / 2;
  let (along, lines, length) = if step.0 == 1 {
    (1, step.1, width)
  } else {
    (step.0, 1, height)
  };
  let count = if step.0 == 1 { height } else { width };
  let mut out = vec![0.0_f32; plane.len()];
  for line in 0..count {
    let base = line * lines;
    for position in 0..length {
      let mut sum = 0.0;
      for (tap, weight) in weights.iter().enumerate() {
        let Some(at) = (position + tap)
          .checked_sub(reach)
          .filter(|at| *at < length)
        else {
          continue;
        };
        sum += plane[base + at * along] * weight;
      }
      out[base + position * along] = sum;
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_shadow_is_the_silhouette_softened_past_its_edge() {
    let cell = (66, 66);
    let margin = shadow_margin(cell) as usize;
    assert_eq!(margin, 11);
    let square = image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
    let shadow = rasterize_shadow(&StoredImage::Still(std::sync::Arc::new(square)), cell)
      .expect("a drawn shadow");
    let side = 66 + 2 * margin;
    let at = |x: usize, y: usize| shadow[(y * side + x) * 4 + 3];
    let middle = side / 2;
    // Solid under the picture, fading across its edge, and gone at the cell's.
    assert!(at(middle, middle) > 240);
    let edge = at(margin + 1, middle);
    assert!((40..215).contains(&edge), "{edge}");
    assert_eq!(at(0, middle), 0);
  }
}
