// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A picture softened the way the spotlights' blur softens a recording, for
//! the live overlay.
//!
//! The overlay has no source to blur on the GPU as the editor does; it only
//! has the desktop captured when a spotlight that blurs is drawn. That still
//! is softened once, here, and the overlay reads the result wherever the blur
//! reaches. It is kept small: averaged down to a fraction of the picture's
//! size, then smoothed with three box passes, which together come close to
//! the Gaussian the editor blurs with at the same width. The overlay
//! stretches it back over the display with bilinear reads.

use crate::screenshots::CapturedImage;

/// How far each box pass reaches either side, in the averaged picture's
/// pixels.
const PASS_RADIUS: usize = 2;

/// `image` softened, at a fraction of its size.
pub(crate) fn soften(image: &CapturedImage) -> CapturedImage {
  let (width, height) = (image.width as usize, image.height as usize);
  if width == 0 || height == 0 || image.rgba.len() < width * height * 4 {
    return CapturedImage {
      rgba: vec![0, 0, 0, 255],
      width: 1,
      height: 1,
    };
  }
  let deviation = super::native::blur_deviation(1.0, image.width, image.height);
  // Three box passes of this radius spread a pixel as a Gaussian of
  // `sqrt(r(r + 1))` averaged pixels does, and the averaging adds a box of
  // one of them, so the step between them is what sets the width.
  let spread = ((PASS_RADIUS * (PASS_RADIUS + 1)) as f32 + 1.0 / 12.0).sqrt();
  let step = ((deviation / spread).round() as usize).max(1);
  let (across, down) = (width.div_ceil(step), height.div_ceil(step));
  let mut small = vec![[0.0_f32; 3]; across * down];
  for (index, pixel) in small.iter_mut().enumerate() {
    let (x0, y0) = ((index % across) * step, (index / across) * step);
    let (x1, y1) = ((x0 + step).min(width), (y0 + step).min(height));
    let mut sum = [0.0_f32; 3];
    for y in y0..y1 {
      for x in x0..x1 {
        let at = (y * width + x) * 4;
        for (total, value) in sum.iter_mut().zip(&image.rgba[at..at + 3]) {
          *total += f32::from(*value);
        }
      }
    }
    let count = ((x1 - x0) * (y1 - y0)) as f32;
    *pixel = sum.map(|value| value / count);
  }
  for _ in 0..3 {
    small = pass(&small, across, down, (1, 0));
    small = pass(&small, across, down, (0, 1));
  }
  CapturedImage {
    rgba: small
      .iter()
      .flat_map(|[red, green, blue]| {
        [red, green, blue]
          .map(|value| value.round().clamp(0.0, 255.0) as u8)
          .into_iter()
          .chain([255])
      })
      .collect(),
    width: across as u32,
    height: down as u32,
  }
}

/// One box pass along `(dx, dy)`, the edge pixel standing in past the edge.
fn pass(
  pixels: &[[f32; 3]],
  across: usize,
  down: usize,
  (dx, dy): (usize, usize),
) -> Vec<[f32; 3]> {
  let span = (2 * PASS_RADIUS + 1) as f32;
  (0..across * down)
    .map(|index| {
      let (x, y) = (index % across, index / across);
      let mut sum = [0.0_f32; 3];
      for offset in 0..=2 * PASS_RADIUS {
        let shifted = |at: usize, delta: usize, limit: usize| {
          (at + offset * delta)
            .saturating_sub(PASS_RADIUS * delta)
            .min(limit - 1)
        };
        let pixel = pixels[shifted(y, dy, down) * across + shifted(x, dx, across)];
        for channel in 0..3 {
          sum[channel] += pixel[channel];
        }
      }
      sum.map(|value| value / span)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stripes_are_softened_to_the_grey_between_them_and_flat_colour_is_kept() {
    let (width, height) = (640_u32, 400_u32);
    let stripes = CapturedImage {
      rgba: (0..height)
        .flat_map(|_| (0..width).flat_map(|x| if x % 2 == 0 { [0, 0, 0, 255] } else { [255; 4] }))
        .collect(),
      width,
      height,
    };
    let soft = soften(&stripes);
    let middle = ((soft.height / 2 * soft.width + soft.width / 2) * 4) as usize;
    assert!(
      soft.rgba[middle].abs_diff(128) <= 2,
      "{}",
      soft.rgba[middle]
    );
    let flat = CapturedImage {
      rgba: [40, 90, 200, 255].repeat((width * height) as usize),
      width,
      height,
    };
    assert_eq!(&soften(&flat).rgba[..4], &[40, 90, 200, 255]);
  }
}
