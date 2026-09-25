// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{inset_colour, surrounding_colour};

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
  assert_eq!(inset_colour(&rgba, 100, 100), Some([34; 3]));
}

#[test]
fn prefers_edges_over_a_large_interior_panel() {
  let mut rgba = frame(100, 100, 34);
  for y in 5..95 {
    for x in 5..95 {
      paint(&mut rgba, 100, x, y, 90);
    }
  }
  assert_eq!(inset_colour(&rgba, 100, 100), Some([34; 3]));
}

#[test]
fn ignores_a_thin_border() {
  let mut rgba = frame(100, 100, 34);
  for i in 0..100 {
    for (x, y) in [(i, 0), (i, 99), (0, i), (99, i)] {
      paint(&mut rgba, 100, x, y, 10);
    }
  }
  assert_eq!(inset_colour(&rgba, 100, 100), Some([34; 3]));
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
  let colour = inset_colour(&rgba, 100, 100).unwrap();
  assert!(colour == [31; 3] || colour == [32; 3], "{colour:?}");
}

#[test]
fn ignores_transparent_rgb_and_handles_small_images() {
  assert_eq!(inset_colour(&[34, 34, 34, 255], 1, 1), Some([34; 3]));
  assert_eq!(inset_colour(&[200, 0, 0, 0], 1, 1), None);
  assert_eq!(inset_colour(&[], 0, 0), None);
  assert_eq!(inset_colour(&[0; 4], 2, 2), None);
}

/// The box is filled with one loud colour that outnumbers the surface, so a
/// ring that strayed inside would vote for it.
#[test]
fn reads_only_the_surface_outside_the_box() {
  let mut rgba = frame(100, 100, 240);
  for y in 20..80 {
    for x in 20..80 {
      paint(&mut rgba, 100, x, y, 7);
    }
  }
  assert_eq!(
    surrounding_colour(&rgba, 100, 100, [20, 20, 80, 80]),
    Some([240; 3])
  );
}

#[test]
fn a_box_against_the_edge_reads_the_sides_that_remain() {
  let mut rgba = frame(100, 100, 240);
  for y in 0..100 {
    for x in 0..30 {
      paint(&mut rgba, 100, x, y, 7);
    }
  }
  // A left side clamped back onto the picture would read the box's own
  // first column, which is the loud colour.
  assert_eq!(
    surrounding_colour(&rgba, 100, 100, [0, 0, 30, 100]),
    Some([240; 3])
  );
}

#[test]
fn a_box_over_the_whole_picture_has_no_surroundings() {
  let rgba = frame(10, 10, 240);
  assert_eq!(surrounding_colour(&rgba, 10, 10, [0, 0, 10, 10]), None);
}
