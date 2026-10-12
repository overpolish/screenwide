// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::compose_screenshot;
use screenwide_output::fixtures::{assert_colour_close, solid_image, test_output_settings};

#[test]
fn inset_stays_distinct_from_the_canvas_background() {
  let mut settings = test_output_settings(400, 400);
  settings.recenter_inset_color = Some("#445566".to_owned());
  settings.image_x = 100.0 - settings.image_width / 2.0;
  let output = compose_screenshot(&solid_image(200, 100, [200, 100, 50, 255]), &settings).unwrap();
  let pixel = |x: u32, y: u32| {
    let start = ((y * output.width + x) * 4) as usize;
    &output.rgba[start..start + 4]
  };
  let assert = assert_colour_close;
  assert(pixel(100, 200), [200, 100, 50, 255]);
  assert(pixel(300, 200), [68, 85, 102, 255]);
  assert(pixel(390, 200), [17, 34, 51, 255]);
}

#[test]
fn source_crop_reveals_inset_without_changing_the_outer_frame() {
  let mut settings = test_output_settings(400, 400);
  settings.recenter_inset_color = Some("#445566".to_owned());
  settings.source_crop.x = 0.5;
  settings.source_crop.width = 0.5;
  let output = compose_screenshot(&solid_image(200, 100, [200, 100, 50, 255]), &settings).unwrap();
  let pixel = |x: u32, y: u32| {
    let start = ((y * output.width + x) * 4) as usize;
    &output.rgba[start..start + 4]
  };
  let assert = assert_colour_close;
  assert(pixel(100, 200), [68, 85, 102, 255]);
  assert(pixel(300, 200), [200, 100, 50, 255]);
  assert(pixel(390, 200), [17, 34, 51, 255]);
}
