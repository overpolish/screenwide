// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{blur_cell, mosaic_cell};

#[test]
fn a_blur_is_as_strong_in_every_box_and_stronger_at_each_step() {
  let cells: Vec<f64> = (1..=5)
    .map(|step| blur_cell(f64::from(step), 1.0))
    .collect();
  assert!(cells.windows(2).all(|pair| pair[0] < pair[1]), "{cells:?}");
  // Sized in output pixels, so a picture drawn at half size doubles them.
  assert_eq!(blur_cell(3.0, 2.0), 2.0 * blur_cell(3.0, 1.0));
}

#[test]
fn classic_pixelation_never_draws_blocks_finer_than_its_smallest() {
  assert_eq!(mosaic_cell(8.0, 1.0), 16.0);
  assert_eq!(mosaic_cell(32.0, 1.0), 32.0);
  assert_eq!(mosaic_cell(24.0, 0.5), 12.0);
}
