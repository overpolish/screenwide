// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The QR codes in a picture: what each holds and where it is, or where a
//! code is that could not be read. rxing reads the codes in one pass over the
//! picture's finder patterns, sampling and decoding each plausible set of
//! three once; rqrr, at the same time, finds every grid that looks like a
//! code, and those rxing did not read are read by rqrr where it can and
//! otherwise marked as codes that could not be read.
//!
//! Its own crate so it can be optimized in dev builds (see the workspace's
//! `Cargo.toml`): both readers are generic over the image, so their code is
//! compiled where it is used, and unoptimized a busy page took seconds.

use image::{DynamicImage, RgbaImage};
use rxing::common::{BitMatrix, DetectorRXingResult, HybridBinarizer};
use rxing::qrcode::cpp_port::{decoder::Decode, detector};
use rxing::{BinaryBitmap, Luma8LuminanceSource};

/// Codes counted as one when this much of the smaller lies in the other.
const DUPLICATE_OVERLAP_THRESHOLD: f64 = 0.45;

/// Where a code is, as fractions of the picture from its top left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QrCode {
  pub bounds: Bounds,
  /// What it holds; none where it looks like a code but could not be read.
  pub content: Option<String>,
}

/// The codes in `rgba`, `width` by `height` pixels, top to bottom and left
/// to right.
pub fn read(rgba: &[u8], width: u32, height: u32) -> Vec<QrCode> {
  let Some(image) = RgbaImage::from_raw(width, height, rgba.to_vec()) else {
    return Vec::new();
  };
  let grayscale = DynamicImage::ImageRgba8(image).into_luma8();
  // Each reader takes its own copy; neither depends on the other until rqrr's
  // grids are compared with what rxing read.
  let for_rxing = grayscale.as_raw().clone();
  let mut codes = std::thread::scope(|scope| {
    let read = scope.spawn(|| decoded_codes(for_rxing, width, height));
    let mut prepared = rqrr::PreparedImage::prepare(grayscale);
    let grids = prepared.detect_grids();
    let mut codes = read
      .join()
      .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    for grid in grids {
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
      if codes
        .iter()
        .any(|code| overlapping_area(code.bounds, bounds) > DUPLICATE_OVERLAP_THRESHOLD)
      {
        continue;
      }
      codes.push(QrCode {
        bounds,
        content: grid.decode().ok().map(|(_, content)| content),
      });
    }
    codes
  });
  codes.sort_by(|a, b| {
    a.bounds
      .y
      .total_cmp(&b.bounds.y)
      .then_with(|| a.bounds.x.total_cmp(&b.bounds.x))
  });
  codes
}

/// What rxing reads. rxing's own search for several codes asks for one
/// code at a time, again in each part of the picture around those found,
/// which on a page of codes takes several times as long and reads no more.
fn decoded_codes(luma: Vec<u8>, width: u32, height: u32) -> Vec<QrCode> {
  let Ok(source) = Luma8LuminanceSource::new(luma, width, height) else {
    return Vec::new();
  };
  let bitmap = BinaryBitmap::new(HybridBinarizer::new(source));
  let matrix: &BitMatrix = bitmap.get_black_matrix();
  let mut patterns = detector::FindFinderPatterns(matrix, true, 0);
  let mut used = Vec::new();
  let mut codes = Vec::new();
  for set in detector::GenerateFinderPatternSets(&mut patterns) {
    if [set.bl, set.tl, set.tr]
      .iter()
      .any(|pattern| used.contains(pattern))
    {
      continue;
    }
    let Ok(sampled) = detector::SampleQR(matrix, &set) else {
      continue;
    };
    let Ok(decoded) = Decode(sampled.getBits()) else {
      continue;
    };
    if !decoded.isValid() {
      continue;
    }
    used.extend([set.bl, set.tl, set.tr]);
    let points = sampled.getPoints().iter().map(|point| (point.x, point.y));
    if let Some(bounds) = normalized_bounds(points, width, height) {
      codes.push(QrCode {
        bounds,
        content: Some(decoded.content().to_string()),
      });
    }
  }
  codes
}

fn normalized_bounds(
  points: impl Iterator<Item = (f32, f32)>,
  width: u32,
  height: u32,
) -> Option<Bounds> {
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
  (right > left && bottom > top).then_some(Bounds {
    height: f64::from(bottom - top) / f64::from(height),
    width: f64::from(right - left) / f64::from(width),
    x: f64::from(left) / f64::from(width),
    y: f64::from(top) / f64::from(height),
  })
}

fn overlapping_area(a: Bounds, b: Bounds) -> f64 {
  let width = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
  let height = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
  if width <= 0.0 || height <= 0.0 {
    return 0.0;
  }
  width * height / (a.width * a.height).min(b.width * b.height)
}

#[cfg(test)]
mod tests;
