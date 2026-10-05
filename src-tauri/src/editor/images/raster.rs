// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's picture drawn at a size, as the image atlas holds it.

use super::StoredImage;

/// `picture` drawn to fill `size` pixels, less a transparent pixel of margin
/// on every side, as premultiplied BGRA rows, top row first. The margin keeps
/// the filtered reads at the picture's edge off the next cell over.
pub(crate) fn rasterize(picture: &StoredImage, size: (u32, u32)) -> Option<Vec<u8>> {
  if size.0 <= 2 || size.1 <= 2 {
    return None;
  }
  match picture {
    StoredImage::Still(image) => Some(framed(image, size)),
    // The atlas draws the frame each moment shows; asked for the picture
    // alone, a moving one is its first.
    StoredImage::Animation(animation) => Some(framed(animation.frames.first()?, size)),
  }
}

/// A stored picture stretched over the cell inside its margin, which its
/// proportions already match, as premultiplied BGRA.
fn framed(image: &image::RgbaImage, size: (u32, u32)) -> Vec<u8> {
  let (inner_width, inner_height) = (size.0 - 2, size.1 - 2);
  let drawn = crate::editor::images::store::resized(image, inner_width, inner_height);
  let mut pixels = vec![0_u8; size.0 as usize * size.1 as usize * 4];
  for (y, row) in drawn.rows().enumerate() {
    let start = ((y + 1) * size.0 as usize + 1) * 4;
    for (out, pixel) in pixels[start..start + inner_width as usize * 4]
      .as_chunks_mut::<4>()
      .0
      .iter_mut()
      .zip(row)
    {
      *out = [pixel[2], pixel[1], pixel[0], pixel[3]];
    }
  }
  pixels
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_picture_fills_its_cell_inside_a_clear_margin() {
    let size = (66, 66);
    let red = image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
    let pixels =
      rasterize(&StoredImage::Still(std::sync::Arc::new(red)), size).expect("a drawn picture");
    let at = |x: usize, y: usize| &pixels[(y * size.0 as usize + x) * 4..][..4];
    // The margin is clear and the picture fills everything inside it, as
    // BGRA.
    assert!((0..66).all(|edge| at(0, edge)[3] == 0 && at(edge, 0)[3] == 0));
    assert_eq!(at(1, 33), [0, 0, 255, 255]);
    assert_eq!(at(33, 33), [0, 0, 255, 255]);
  }
}
