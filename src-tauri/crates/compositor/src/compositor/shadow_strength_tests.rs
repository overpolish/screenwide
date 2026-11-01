// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{compositor, draw, read_pixel, target, white_source, OUTPUT};
use super::*;
use screenwide_output::fixtures::test_output_settings;

/// An 800px-square screen in the middle of the canvas, casting its shadow
/// as strong as `strength` says.
fn shadowed(strength: Option<f64>) -> ScreenshotOutputSettings {
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  settings.drop_shadow = true;
  settings.drop_shadow_strength = strength;
  settings.image_width = 800.0;
  settings.image_x = 560.0;
  settings.image_y = 140.0;
  settings.crop_width = 800.0;
  settings.crop_height = 800.0;
  settings.crop_x = 560.0;
  settings.crop_y = 140.0;
  settings
}

fn brightness(pixel: [u8; 4]) -> i32 {
  pixel[..3].iter().map(|channel| i32::from(*channel)).sum()
}

#[test]
#[ignore = "requires a GPU adapter"]
fn a_shadow_fading_in_is_drawn_part_as_dark() {
  let compositor = compositor();
  let source = white_source(&compositor, (64, 64));
  let brightness_at = |settings: &ScreenshotOutputSettings| {
    let texture = target();
    draw(&compositor, &texture, &source, settings);
    // Just past the screen's bottom edge, where its shadow falls.
    brightness(read_pixel(&texture, 960, 950))
  };
  let mut unshadowed = shadowed(None);
  unshadowed.drop_shadow = false;
  let canvas = brightness_at(&unshadowed);
  let whole = canvas - brightness_at(&shadowed(None));
  let half = canvas - brightness_at(&shadowed(Some(0.5)));
  let none = canvas - brightness_at(&shadowed(Some(0.0)));
  assert!(
    whole > 8,
    "a whole shadow should darken the canvas, by {whole}"
  );
  assert!(
    (half * 2 - whole).abs() <= 4,
    "half a shadow should darken it half as much: {half} of {whole}"
  );
  assert!(
    none.abs() <= 1,
    "no shadow should leave it as it was, by {none}"
  );
}
