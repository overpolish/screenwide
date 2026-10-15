// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A scene that turns the screen in depth draws it where the turn carries
//! it, the canvas showing through where the upright screen stood.

use super::render_test_helpers::{compositor, draw, read_pixel, target};
use super::*;
use screenwide_imaging::CapturedImage;
use screenwide_output::fixtures::test_output_settings;
use screenwide_scenes::{SceneMotion, SceneRotation};

/// A white canvas with a black 400 by 300 screen in its middle.
fn settings(rotation: Option<SceneRotation>) -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(1920, 1080);
  settings.background_color = "#ffffff".to_owned();
  (
    settings.image_width,
    settings.crop_width,
    settings.crop_height,
  ) = (400.0, 400.0, 300.0);
  (
    settings.image_x,
    settings.image_y,
    settings.crop_x,
    settings.crop_y,
  ) = (760.0, 390.0, 760.0, 390.0);
  settings.scene_rotation = rotation;
  settings
}

fn brightness(pixel: [u8; 4]) -> u32 {
  pixel[..3]
    .iter()
    .map(|value| u32::from(*value))
    .sum::<u32>()
    / 3
}

/// How bright the canvas is drawn at each of `points` with the screen turned
/// by `rotation`.
fn drawn(rotation: Option<SceneRotation>, points: &[(u32, u32)]) -> Vec<u32> {
  drawn_with(settings(rotation), points)
}

fn drawn_with(settings: ScreenshotOutputSettings, points: &[(u32, u32)]) -> Vec<u32> {
  let compositor = compositor();
  let source = compositor
    .screenshot_source(&CapturedImage {
      width: 400,
      height: 300,
      rgba: [0, 0, 0, 255].repeat(400 * 300),
    })
    .expect("the source uploads");
  let output = target();
  draw(&compositor, &output, &source, &settings);
  points
    .iter()
    .map(|&(x, y)| brightness(read_pixel(&output, x, y)))
    .collect()
}

#[test]
fn a_screen_turned_away_leaves_the_canvas_where_its_far_side_stood() {
  // Just inside the upright screen's right edge, and its middle.
  let points = [(1_150, 540), (960, 540)];
  assert_eq!(drawn(None, &points), [0, 0]);
  let turned = drawn(
    Some(SceneRotation {
      x: 0.0,
      y: 60.0,
      z: 0.0,
    }),
    &points,
  );
  assert!(
    turned[0] > 250,
    "the far side still covers the canvas there"
  );
  assert_eq!(turned[1], 0, "the middle stays where it stood");
}

#[test]
fn a_quarter_turn_stands_the_screen_on_its_side() {
  // Above the upright screen's top edge, inside the turned one's.
  let points = [(960, 360)];
  assert!(drawn(None, &points)[0] > 250);
  let turned = drawn(
    Some(SceneRotation {
      x: 0.0,
      y: 0.0,
      z: 90.0,
    }),
    &points,
  );
  assert_eq!(turned[0], 0);
}

#[test]
fn a_turn_made_while_the_shutter_was_open_is_blurred_along_its_way() {
  // Above the upright screen's top edge, inside the screen stood on its
  // side: the turn sweeps over it part of the way from upright.
  let points = [(960, 360)];
  let mut turning = settings(Some(SceneRotation {
    x: 0.0,
    y: 0.0,
    z: 90.0,
  }));
  turning.scene_motion = Some(SceneMotion {
    screen: [1.0, 0.0, 0.0],
    image: [1.0, 0.0, 0.0],
    camera_frame: [0.0; 4],
    camera_crop: [0.0, 0.0, 1.0, 1.0],
    rotation: SceneRotation::UPRIGHT,
    samples: 24,
  });
  let blurred = drawn_with(turning, &points)[0];
  assert!(
    (40..=215).contains(&blurred),
    "the turn is drawn sharp there: {blurred}"
  );
}
