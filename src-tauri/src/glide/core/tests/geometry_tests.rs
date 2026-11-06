// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use crate::glide::{
  core::{
    corrected_origin, frame_fits, frame_fractions, frames_match, grown_to_min, landing_point,
    GlideFrame,
  },
  region_rect::{Gravity, RegionGravity},
};

fn gravity(horizontal: Gravity, vertical: Gravity) -> RegionGravity {
  RegionGravity {
    horizontal,
    vertical,
  }
}

#[test]
fn a_left_half_below_the_minimum_keeps_its_left_edge() {
  // A 1280-point display: half is 640, the editor's minimum frame is 800x628.
  assert_eq!(
    grown_to_min(
      frame(0.0, 25.0, 640.0, 875.0),
      (800.0, 628.0),
      gravity(Gravity::Start, Gravity::Center),
    ),
    frame(0.0, 25.0, 800.0, 875.0)
  );
}

#[test]
fn a_right_half_below_the_minimum_keeps_its_right_edge() {
  assert_eq!(
    grown_to_min(
      frame(640.0, 25.0, 640.0, 875.0),
      (800.0, 628.0),
      gravity(Gravity::End, Gravity::Center),
    ),
    frame(480.0, 25.0, 800.0, 875.0)
  );
}

#[test]
fn a_quarter_below_the_minimum_grows_away_from_its_corner_on_both_axes() {
  assert_eq!(
    grown_to_min(
      frame(640.0, 462.0, 640.0, 438.0),
      (800.0, 628.0),
      gravity(Gravity::End, Gravity::End),
    ),
    frame(480.0, 272.0, 800.0, 628.0)
  );
}

#[test]
fn a_centered_region_below_the_minimum_grows_about_its_middle() {
  assert_eq!(
    grown_to_min(
      frame(400.0, 25.0, 480.0, 875.0),
      (800.0, 628.0),
      gravity(Gravity::Center, Gravity::Center),
    ),
    frame(240.0, 25.0, 800.0, 875.0)
  );
}

#[test]
fn a_region_at_or_above_the_minimum_is_left_alone() {
  let region = frame(13.5, 25.0, 800.0, 875.0);
  assert_eq!(
    grown_to_min(region, (800.0, 628.0), gravity(Gravity::End, Gravity::End)),
    region
  );
}

#[test]
fn matching_frames_include_position_and_size_with_rounding_tolerance() {
  let original = frame(10.0, 20.0, 800.0, 600.0);
  assert!(frames_match(original, frame(10.5, 19.5, 800.5, 599.5), 1.0));
  assert!(!frames_match(
    original,
    frame(12.0, 20.0, 800.0, 600.0),
    1.0
  ));
}

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
