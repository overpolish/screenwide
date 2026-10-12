// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A person cut out of their background casts their drop shadow from their
//! outline: under them, and not along the edge of a frame that shows
//! nothing, whether they are the camera over the screen or the picture
//! itself.

use super::render_test_helpers::{compositor, read_pixel, target, view};
use super::*;
use screenwide_imaging::CapturedImage;
use screenwide_output::fixtures::test_output_settings;

/// A 200 pixel square with nothing in it but a disc of radius 50 at its
/// middle, as a person cut out of their room.
fn disc() -> CapturedImage {
  let mut rgba = vec![0; 200 * 200 * 4];
  for (index, pixel) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
    let (x, y) = ((index % 200) as f32 - 99.5, (index / 200) as f32 - 99.5);
    if x.hypot(y) < 50.0 {
      *pixel = [200, 60, 60, 255];
    }
  }
  CapturedImage {
    width: 200,
    height: 200,
    rgba,
  }
}

fn brightness(pixel: [u8; 4]) -> u32 {
  pixel[..3]
    .iter()
    .map(|value| u32::from(*value))
    .sum::<u32>()
    / 3
}

/// White canvas settings with Drop shadow on and the screen's picture at
/// (800, 400), 200 pixels square.
fn settings() -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(1920, 1080);
  settings.background_color = "#ffffff".to_owned();
  settings.drop_shadow = true;
  (
    settings.image_width,
    settings.crop_width,
    settings.crop_height,
  ) = (200.0, 200.0, 200.0);
  (
    settings.image_x,
    settings.image_y,
    settings.crop_x,
    settings.crop_y,
  ) = (800.0, 400.0, 800.0, 400.0);
  settings
}

fn frame() -> ComposedFrame {
  ComposedFrame {
    cursor: None,
    keyboard: None,
    foreground_only: false,
    seconds: 0.0,
  }
}

/// Just below the disc is in its shadow; just below the bottom edge of the
/// square, with the disc well above, is the canvas untouched.
fn assert_outline_shadow(output: &wgpu::Texture, (left, top): (u32, u32), what: &str) {
  let under = brightness(read_pixel(output, left + 100, top + 156));
  assert!(
    under < 245,
    "{what}: the person casts no shadow below them, was {under}"
  );
  let edge = brightness(read_pixel(output, left + 100, top + 205));
  assert!(
    edge >= 253,
    "{what}: the empty frame casts a shadow, was {edge}"
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_cut_out_screen_picture_casts_its_shadow_from_the_person() {
  let compositor = compositor();
  let source = SourceTexture {
    cut_out: true,
    ..compositor.screenshot_source(&disc()).unwrap()
  };
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings(),
      frame(),
      None,
      None,
      &Default::default(),
    )
    .unwrap();
  assert_outline_shadow(&output, (800, 400), "the picture");
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_cut_out_picture_filling_the_canvas_still_casts_its_shadow() {
  let compositor = compositor();
  let source = SourceTexture {
    cut_out: true,
    ..compositor.screenshot_source(&disc()).unwrap()
  };
  // Full height, so no canvas is left above or below it: a frame's shadow
  // would have no room, but the person has room inside the frame.
  let mut settings = settings();
  (
    settings.image_width,
    settings.crop_width,
    settings.crop_height,
  ) = (1080.0, 1080.0, 1080.0);
  (
    settings.image_x,
    settings.image_y,
    settings.crop_x,
    settings.crop_y,
  ) = (420.0, 0.0, 420.0, 0.0);
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings,
      frame(),
      None,
      None,
      &Default::default(),
    )
    .unwrap();
  // The disc's bottom edge is at 810, its radius 270 pixels.
  let under = brightness(read_pixel(&output, 960, 830));
  assert!(
    under < 245,
    "the person casts no shadow below them, was {under}"
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_cut_out_camera_casts_its_shadow_from_the_person() {
  let compositor = compositor();
  let screen = compositor
    .screenshot_source(&CapturedImage {
      width: 2,
      height: 2,
      rgba: vec![255; 16],
    })
    .unwrap();
  let camera = SourceTexture {
    cut_out: true,
    ..compositor.screenshot_source(&disc()).unwrap()
  };
  let mut settings = settings();
  settings.drop_shadow = false;
  let geometry = BakeGeometry {
    crop_height: 200,
    crop_width: 200,
    crop_x: 0,
    crop_y: 0,
    frame_height: 200,
    frame_width: 200,
    frame_x: 300,
    frame_y: 300,
    output_height: 1080,
    output_width: 1920,
    radius: 0,
  };
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &screen,
      &settings,
      frame(),
      Some((&camera, geometry, true, true)),
      None,
      &Default::default(),
    )
    .unwrap();
  assert_outline_shadow(&output, (300, 300), "the camera");
}
