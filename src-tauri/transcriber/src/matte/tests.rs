// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::model::{planar, quantise};

#[test]
fn planar_splits_interleaved_pixels_into_channel_planes() {
  let rgb = [255, 0, 51, 0, 102, 255];
  let mut planes = [0.0; 6];
  planar(&rgb, 2, &mut planes);
  assert_eq!(planes, [1.0, 0.0, 0.0, 0.4, 0.2, 1.0]);
}

#[test]
fn quantise_clamps_the_models_overshoot() {
  let mut bytes = [0; 4];
  quantise(&[-0.2, 0.5, 1.0, 1.3], &mut bytes);
  assert_eq!(bytes, [0, 128, 255, 255]);
}
