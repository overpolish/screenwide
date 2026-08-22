// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn view(rgba: &[u8], width: u32, height: u32) -> ImageView<'_> {
  ImageView {
    height,
    rgba,
    width,
  }
}

fn whole(width: u32, height: u32) -> ImageRegion {
  ImageRegion {
    height,
    width,
    x: 0,
    y: 0,
  }
}

#[test]
fn finds_a_dominant_background_behind_a_smaller_foreground_patch() {
  let background = [29, 31, 30, 255];
  let mut rgba = background.repeat(100);
  for pixel in rgba.chunks_exact_mut(4).take(20) {
    pixel.copy_from_slice(&[220, 80, 40, 255]);
  }

  let sample = detect_background(view(&rgba, 10, 10), whole(10, 10), 1).unwrap();

  assert_eq!(sample.colour, background);
  assert_eq!(sample.samples, 100);
  assert!((sample.confidence - 0.8).abs() < f32::EPSILON);
}

#[test]
fn averages_noise_within_the_winning_colour_bucket() {
  let rgba = [[32, 33, 34, 255], [35, 36, 37, 255], [240, 20, 20, 255]].concat();

  let sample = detect_background(view(&rgba, 3, 1), whole(3, 1), 1).unwrap();

  assert_eq!(sample.colour, [33, 34, 35, 255]);
  assert_eq!(sample.samples, 3);
}

#[test]
fn region_and_predicate_limit_which_pixels_can_vote() {
  let rgba = [
    [10, 10, 10, 255],
    [200, 200, 200, 255],
    [32, 32, 32, 255],
    [40, 40, 40, 255],
  ]
  .concat();

  let sample = detect_background_where(
    view(&rgba, 4, 1),
    ImageRegion {
      height: 1,
      width: 3,
      x: 1,
      y: 0,
    },
    1,
    |x, _| x != 1,
  )
  .unwrap();

  assert_eq!(sample.colour, [36, 36, 36, 255]);
  assert_eq!(sample.samples, 2);
}

#[test]
fn rejects_empty_out_of_bounds_and_truncated_images() {
  assert!(detect_background(view(&[], 0, 0), whole(0, 0), 1).is_none());
  assert!(detect_background(view(&[0; 12], 2, 2), whole(2, 2), 1).is_none());
  assert!(detect_background(
    view(&[0; 16], 2, 2),
    ImageRegion {
      height: 1,
      width: 1,
      x: 3,
      y: 0,
    },
    1,
  )
  .is_none());
}
