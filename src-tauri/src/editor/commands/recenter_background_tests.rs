// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::detect;

fn frame(width: usize, height: usize, colour: u8) -> Vec<u8> {
  [colour, colour, colour, 255].repeat(width * height)
}

fn paint(rgba: &mut [u8], width: usize, x: usize, y: usize, colour: u8) {
  rgba[(y * width + x) * 4..][..3].fill(colour);
}

#[test]
fn preserves_dark_background_with_nearby_controls() {
  let mut rgba = frame(100, 100, 34);
  for y in 10..100 {
    for x in 70..100 {
      paint(&mut rgba, 100, x, y, 44);
    }
  }
  assert_eq!(detect(&rgba, 100, 100), Some([34; 3]));
}

#[test]
fn prefers_edges_over_a_large_interior_panel() {
  let mut rgba = frame(100, 100, 34);
  for y in 5..95 {
    for x in 5..95 {
      paint(&mut rgba, 100, x, y, 90);
    }
  }
  assert_eq!(detect(&rgba, 100, 100), Some([34; 3]));
}

#[test]
fn ignores_a_thin_border() {
  let mut rgba = frame(100, 100, 34);
  for i in 0..100 {
    for (x, y) in [(i, 0), (i, 99), (0, i), (99, i)] {
      paint(&mut rgba, 100, x, y, 10);
    }
  }
  assert_eq!(detect(&rgba, 100, 100), Some([34; 3]));
}

#[test]
fn groups_noise_across_old_bucket_boundaries() {
  let mut rgba = frame(100, 100, 31);
  for y in 0..100 {
    for x in 0..100 {
      paint(
        &mut rgba,
        100,
        x,
        y,
        if x < 30 {
          90
        } else if y % 2 == 0 {
          31
        } else {
          32
        },
      );
    }
  }
  let colour = detect(&rgba, 100, 100).unwrap();
  assert!(colour == [31; 3] || colour == [32; 3], "{colour:?}");
}

#[test]
fn ignores_transparent_rgb_and_handles_small_images() {
  assert_eq!(detect(&[34, 34, 34, 255], 1, 1), Some([34; 3]));
  assert_eq!(detect(&[200, 0, 0, 0], 1, 1), None);
  assert_eq!(detect(&[], 0, 0), None);
  assert_eq!(detect(&[0; 4], 2, 2), None);
}
