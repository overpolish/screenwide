// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use super::*;

/// A still with `colours` distinct colours, tiled over the whole image.
fn palette_image(width: u32, height: u32, colours: u32, alpha: u8) -> CapturedImage {
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for pixel in 0..width * height {
    let colour = pixel % colours;
    rgba.extend_from_slice(&[
      (colour % 256) as u8,
      (colour / 256 % 256) as u8,
      (colour / 65_536 % 256) as u8,
      alpha,
    ]);
  }
  CapturedImage {
    rgba,
    width,
    height,
  }
}

/// A smooth gradient, which is far past what a 256-colour palette holds.
fn gradient(width: u32, height: u32, alpha: u8) -> CapturedImage {
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    for x in 0..width {
      rgba.extend_from_slice(&[(x % 256) as u8, (y % 256) as u8, 128, alpha]);
    }
  }
  CapturedImage {
    rgba,
    width,
    height,
  }
}

fn png_info(png: &[u8]) -> (png::ColorType, png::BitDepth, u32, u32, usize) {
  let reader = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
  let info = reader.info();
  let palette = info.palette.as_ref().map_or(0, |palette| palette.len() / 3);
  (
    info.color_type,
    info.bit_depth,
    info.width,
    info.height,
    palette,
  )
}

#[test]
fn writes_an_opaque_still_as_an_eight_bit_indexed_png() {
  let image = gradient(320, 200, 255);
  let png = encode_png(&image).unwrap();
  let (color, depth, width, height, palette) = png_info(&png);
  assert_eq!(color, png::ColorType::Indexed);
  assert_eq!(depth, png::BitDepth::Eight);
  assert_eq!((width, height), (320, 200));
  assert!(palette > 0 && palette <= MAX_PALETTE);
  assert!(png.len() < image.rgba.len());
}

#[test]
fn keeps_a_still_that_already_fits_a_palette_bit_exact() {
  let image = palette_image(64, 64, 200, 255);
  let png = encode_png(&image).unwrap();
  let (color, _, _, _, palette) = png_info(&png);
  assert_eq!(color, png::ColorType::Indexed);
  assert_eq!(palette, 200);
  let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
  assert_eq!(decoded.into_raw(), image.rgba);
}

#[test]
fn stays_close_to_the_original_when_it_has_to_quantize() {
  let image = gradient(256, 256, 255);
  let png = encode_png(&image).unwrap();
  let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
  assert_eq!(decoded.dimensions(), (256, 256));
  let error: u64 = decoded
    .as_raw()
    .iter()
    .zip(&image.rgba)
    .map(|(decoded, original)| u64::from(decoded.abs_diff(*original)))
    .sum();
  let mean = error as f64 / image.rgba.len() as f64;
  assert!(mean < 8.0, "mean channel error {mean} is too high");
}

#[test]
fn keeps_every_channel_when_the_still_is_not_opaque() {
  let image = gradient(64, 64, 128);
  let png = encode_png(&image).unwrap();
  let (color, _, _, _, _) = png_info(&png);
  assert_eq!(color, png::ColorType::Rgba);
  let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
  assert_eq!(decoded.into_raw(), image.rgba);
}

#[test]
fn rounds_only_the_requested_corners_with_antialiasing() {
  let image = palette_image(40, 20, 1, 255);
  let rounded = rounded_corners(&image, 50.0);
  let alpha = |x: u32, y: u32| rounded.rgba[((y * rounded.width + x) * 4 + 3) as usize];
  assert_eq!(alpha(0, 0), 0);
  assert_eq!(alpha(20, 10), 255);
  assert!(alpha(2, 3) > 0 && alpha(2, 3) < 255);
  assert_eq!(rounded.width, image.width);
  assert_eq!(rounded.height, image.height);
}
