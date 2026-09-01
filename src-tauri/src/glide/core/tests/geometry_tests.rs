// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::glide::{
  core::{corrected_origin, frame_fits, frame_fractions, landing_point, GlideFrame},
  region_rect::{Gravity, RegionGravity},
};

fn frame(x: f64, y: f64, width: f64, height: f64) -> GlideFrame {
  GlideFrame {
    x,
    y,
    width,
    height,
  }
}

#[test]
fn landing_keeps_the_proportional_grip_and_titlebar_depth() {
  assert_eq!(
    landing_point(
      (300.0, 60.0),
      frame(100.0, 50.0, 800.0, 600.0),
      frame(960.0, 25.0, 400.0, 500.0),
    ),
    (1_060.0, 35.0)
  );
}

#[test]
fn landing_clamps_a_grip_into_the_achieved_frame() {
  assert_eq!(
    landing_point(
      (1_000.0, 700.0),
      frame(100.0, 50.0, 800.0, 600.0),
      frame(0.0, 0.0, 200.0, 100.0),
    ),
    (200.0, 100.0)
  );
}

#[test]
fn fit_tolerates_device_pixel_rounding() {
  assert!(frame_fits(
    frame(0.0, 0.0, 718.5, 876.0),
    frame(0.0, 0.0, 720.0, 875.0),
    2.0,
  ));
}

#[test]
fn constrained_frames_follow_the_regions_gravity() {
  assert_eq!(
    corrected_origin(
      frame(-720.0, 25.0, 720.0, 875.0),
      frame(0.0, 0.0, 500.0, 875.0),
      RegionGravity {
        horizontal: Gravity::End,
        vertical: Gravity::Center,
      },
    ),
    (-500.0, 25.0)
  );
}

#[test]
fn frames_project_into_work_area_fractions() {
  assert_eq!(
    frame_fractions(
      frame(1_080.0, 450.0, 360.0, 450.0),
      (0.0, 0.0),
      (1_440.0, 900.0),
    ),
    Some(frame(0.75, 0.5, 0.25, 0.5))
  );
}
