// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use image::{imageops::FilterType, GrayImage, Luma, RgbaImage};
use qrcode::QrCode;

use super::recognize;

const URL: &str = "https://screenwide.app/robustness";

fn qr_image(content: &str, size: u32) -> GrayImage {
  QrCode::new(content)
    .unwrap()
    .render::<Luma<u8>>()
    .quiet_zone(true)
    .min_dimensions(size, size)
    .build()
}

fn recognize_image(image: &GrayImage) -> Vec<super::RecognizedQrCode> {
  let rgba = RgbaImage::from_fn(image.width(), image.height(), |x, y| {
    let value = image.get_pixel(x, y).0[0];
    image::Rgba([value, value, value, 255])
  });
  recognize(rgba.as_raw(), rgba.width(), rgba.height())
}

fn assert_decodes(image: &GrayImage, expected: &str) {
  let results = recognize_image(image);
  let result = results
    .iter()
    .find(|result| result.content == expected)
    .unwrap_or_else(|| panic!("did not decode {expected}; results: {results:?}"));
  assert!(result.decode_error.is_none());
  assert!(result.bounds.width > 0.0 && result.bounds.height > 0.0);
  assert!(result.bounds.x >= 0.0 && result.bounds.y >= 0.0);
  assert!(result.bounds.x + result.bounds.width <= 1.0);
  assert!(result.bounds.y + result.bounds.height <= 1.0);
}

fn sample(image: &GrayImage, x: f32, y: f32) -> Luma<u8> {
  if x < 0.0 || y < 0.0 || x >= image.width() as f32 || y >= image.height() as f32 {
    Luma([255])
  } else {
    *image.get_pixel(
      (x.round() as u32).min(image.width() - 1),
      (y.round() as u32).min(image.height() - 1),
    )
  }
}

fn rotate(image: &GrayImage, degrees: f32) -> GrayImage {
  let padding = 48;
  let width = image.width() + padding * 2;
  let height = image.height() + padding * 2;
  let radians = degrees.to_radians();
  let (sin, cos) = radians.sin_cos();
  let source_center = (image.width() as f32 / 2.0, image.height() as f32 / 2.0);
  let target_center = (width as f32 / 2.0, height as f32 / 2.0);
  GrayImage::from_fn(width, height, |x, y| {
    let dx = x as f32 - target_center.0;
    let dy = y as f32 - target_center.1;
    sample(
      image,
      cos * dx + sin * dy + source_center.0,
      -sin * dx + cos * dy + source_center.1,
    )
  })
}

fn keystone(image: &GrayImage, top_scale: f32) -> GrayImage {
  let padding = 40;
  let width = image.width() + padding * 2;
  let height = image.height() + padding * 2;
  GrayImage::from_fn(width, height, |x, y| {
    let source_y = y as f32 - padding as f32;
    let progress = (source_y / image.height() as f32).clamp(0.0, 1.0);
    let scale = top_scale + (1.0 - top_scale) * progress;
    let target_center = width as f32 / 2.0;
    let source_x = (x as f32 - target_center) / scale + image.width() as f32 / 2.0;
    sample(image, source_x, source_y)
  })
}

#[test]
fn decodes_moderate_perspective_keystone() {
  assert_decodes(&keystone(&qr_image(URL, 240), 0.9), URL);
}

#[test]
fn severe_perspective_is_unsupported_not_a_false_payload() {
  let results = recognize_image(&keystone(&qr_image(URL, 240), 0.78));
  assert!(!results.iter().any(|result| result.content == URL));
  assert!(results.iter().any(|result| result.decode_error.is_some()));
}

#[test]
fn decodes_non_uniform_stretch() {
  let source = qr_image(URL, 220);
  let stretched = image::imageops::resize(
    &source,
    source.width() * 3 / 2,
    source.height() * 3 / 4,
    FilterType::Nearest,
  );
  assert_decodes(&stretched, URL);
}

#[test]
fn decodes_rotation() {
  assert_decodes(&rotate(&qr_image(URL, 220), 14.0), URL);
}

#[test]
fn decodes_small_modules() {
  let source = qr_image(URL, 220);
  let small = image::imageops::resize(&source, 96, 96, FilterType::Triangle);
  assert_decodes(&small, URL);
}

#[test]
fn decodes_mild_blur_and_reduced_contrast() {
  let source = qr_image(URL, 220);
  let blurred = image::imageops::blur(&source, 1.1);
  let low_contrast = image::imageops::contrast(&blurred, -25.0);
  assert_decodes(&low_contrast, URL);
}

#[test]
fn decodes_multiple_transformed_codes() {
  let urls = [
    "https://screenwide.app/perspective",
    "https://screenwide.app/stretched",
  ];
  let first = keystone(&qr_image(urls[0], 190), 0.82);
  let second_source = qr_image(urls[1], 190);
  let second = image::imageops::resize(
    &second_source,
    second_source.width() * 5 / 4,
    second_source.height() * 4 / 5,
    FilterType::Nearest,
  );
  let mut canvas = GrayImage::from_pixel(
    first.width() + second.width() + 40,
    first.height().max(second.height()) + 40,
    Luma([255]),
  );
  image::imageops::overlay(&mut canvas, &first, 10, 20);
  image::imageops::overlay(&mut canvas, &second, i64::from(first.width() + 30), 20);
  let contents = recognize_image(&canvas)
    .into_iter()
    .map(|result| result.content)
    .collect::<HashSet<_>>();
  assert!(HashSet::from(urls.map(str::to_owned)).is_subset(&contents));
}

#[test]
fn corrupted_code_is_never_returned_as_a_false_payload() {
  let mut corrupted = qr_image(URL, 220);
  for y in corrupted.height() / 3..corrupted.height() * 2 / 3 {
    for x in corrupted.width() / 3..corrupted.width() * 2 / 3 {
      corrupted.put_pixel(x, y, Luma([255]));
    }
  }
  let results = recognize_image(&corrupted);
  assert!(!results.iter().any(|result| result.content == URL));
  assert!(results.iter().any(|result| result.decode_error.is_some()));
}
