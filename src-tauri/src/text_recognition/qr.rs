// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use image::{DynamicImage, GrayImage, RgbaImage};
use rxing::{BarcodeFormat, DecodeHints};

use super::{RecognizedQrCode, TextRect};

const DECODE_ERROR: &str = "QR-like code could not be decoded.";
const DUPLICATE_OVERLAP_THRESHOLD: f64 = 0.45;

pub(super) fn recognize(rgba: &[u8], width: u32, height: u32) -> Vec<RecognizedQrCode> {
  let Some(image) = RgbaImage::from_raw(width, height, rgba.to_vec()) else {
    return Vec::new();
  };
  let grayscale = DynamicImage::ImageRgba8(image).into_luma8();
  let mut results = decoded_codes(&grayscale);
  add_undecoded_candidates(&mut results, grayscale);
  results.sort_by(|a, b| {
    a.bounds
      .y
      .total_cmp(&b.bounds.y)
      .then_with(|| a.bounds.x.total_cmp(&b.bounds.x))
  });
  results
}

fn decoded_codes(image: &GrayImage) -> Vec<RecognizedQrCode> {
  let mut hints = DecodeHints {
    PossibleFormats: Some(HashSet::from([BarcodeFormat::QR_CODE])),
    ..DecodeHints::default()
  };
  rxing::helpers::detect_multiple_in_luma_with_hints(
    image.as_raw().clone(),
    image.width(),
    image.height(),
    &mut hints,
  )
  .unwrap_or_default()
  .into_iter()
  .filter_map(|result| {
    normalized_bounds(
      result.getPoints().iter().map(|point| (point.x, point.y)),
      image.width(),
      image.height(),
    )
    .map(|bounds| RecognizedQrCode {
      bounds,
      content: result.getText().to_owned(),
      decode_error: None,
    })
  })
  .collect()
}

fn add_undecoded_candidates(results: &mut Vec<RecognizedQrCode>, image: GrayImage) {
  let width = image.width();
  let height = image.height();
  let mut prepared = rqrr::PreparedImage::prepare(image);
  for grid in prepared.detect_grids() {
    let Some(bounds) = normalized_bounds(
      grid
        .bounds
        .iter()
        .map(|point| (point.x as f32, point.y as f32)),
      width,
      height,
    ) else {
      continue;
    };
    if results
      .iter()
      .any(|result| overlapping_area(result.bounds, bounds) > DUPLICATE_OVERLAP_THRESHOLD)
    {
      continue;
    }
    let (content, decode_error) = grid.decode().map_or_else(
      |_| (String::new(), Some(DECODE_ERROR.to_owned())),
      |(_, content)| (content, None),
    );
    results.push(RecognizedQrCode {
      bounds,
      content,
      decode_error,
    });
  }
}

fn normalized_bounds(
  points: impl Iterator<Item = (f32, f32)>,
  width: u32,
  height: u32,
) -> Option<TextRect> {
  if width == 0 || height == 0 {
    return None;
  }
  let mut left = f32::INFINITY;
  let mut top = f32::INFINITY;
  let mut right = f32::NEG_INFINITY;
  let mut bottom = f32::NEG_INFINITY;
  for (x, y) in points {
    left = left.min(x);
    top = top.min(y);
    right = right.max(x);
    bottom = bottom.max(y);
  }
  left = left.clamp(0.0, width as f32);
  top = top.clamp(0.0, height as f32);
  right = right.clamp(0.0, width as f32);
  bottom = bottom.clamp(0.0, height as f32);
  (right > left && bottom > top).then_some(TextRect {
    height: f64::from(bottom - top) / f64::from(height),
    width: f64::from(right - left) / f64::from(width),
    x: f64::from(left) / f64::from(width),
    y: f64::from(top) / f64::from(height),
  })
}

fn overlapping_area(a: TextRect, b: TextRect) -> f64 {
  let width = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
  let height = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
  if width <= 0.0 || height <= 0.0 {
    return 0.0;
  }
  width * height / (a.width * a.height).min(b.width * b.height)
}

#[cfg(test)]
#[path = "qr_tests.rs"]
mod tests;
