// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn a_picture_is_drawn_at_the_doubling_above_its_drawn_size() {
  assert_eq!(raster_side(100.0, 1.0), 128);
  // A Retina display at the canvas's own size draws it twice as large.
  assert_eq!(raster_side(100.0, 0.5), 256);
  assert_eq!(raster_side(2.0, 1.0), MIN_RASTER);
  assert_eq!(raster_side(100_000.0, 1.0), MAX_RASTER);
  assert_eq!(raster_side(0.0, 1.0), 0);
}

#[test]
fn a_still_is_never_drawn_larger_than_its_own_picture() {
  // A 1440 px upscaled picture shown at 2048 px is drawn at 1440 and
  // stretched by the shader; shown smaller, it is drawn at its doubling.
  assert_eq!(still_side(2_048, 1_440), 1_440);
  assert_eq!(still_side(512, 1_440), 512);
  assert_eq!(still_side(64, 8), MIN_RASTER);
}

#[test]
fn a_cell_keeps_the_pictures_proportions_inside_its_margin() {
  assert_eq!(cell_of([200.0, 100.0], 128), Some((130, 66)));
  assert_eq!(cell_of([50.0, 100.0], 64), Some((34, 66)));
  assert_eq!(cell_of([0.0, 100.0], 64), None);
}
