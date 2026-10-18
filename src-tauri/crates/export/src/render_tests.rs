// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stills composed through the shared compositor, pixel by pixel: every
//! annotation kind, the cursor's order among them and what a redaction keeps
//! hidden. They run wherever the compositor does.

mod annotation_tests;
mod counter_tests;
mod cursor_order_tests;
mod draw_tests;
mod image_tests;
mod magnify_tests;
mod redact_attack_tests;
mod redact_blur_tests;
mod redact_classic_tests;
mod redact_tests;
mod shape_tests;
mod spotlight_tests;
mod text_tests;

/// A plain `width` by `height` canvas: black, uncropped, the picture filling it.
pub(crate) fn output(width: u32, height: u32) -> screenwide_output::ScreenshotOutputSettings {
  let mut output = screenwide_output::fixtures::test_output_settings(width, height);
  output.background_color = "#000000".to_owned();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.mesh_seed = 0;
  output.mesh_warp_percent = 0.0;
  output.crop_height = f64::from(height);
  output.crop_width = f64::from(width);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(width);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output
}
