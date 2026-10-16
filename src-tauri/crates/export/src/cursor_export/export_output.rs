// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas a scaled export draws on. The look is measured at the output's
//! reference size; an export at another resolution carries it, and what it
//! placed on it, onto the smaller or larger canvas before drawing.

use screenwide_compositor::compositor::BakeGeometry;
use screenwide_output::ScreenshotOutputSettings;

use super::CursorExportRequest;

/// `request`'s output settings at the resolution its video options ask for.
pub(super) fn scaled(request: &CursorExportRequest<'_>) -> ScreenshotOutputSettings {
  let mut output = request.output.clone();
  (output.width, output.height) =
    super::output_dimensions(output.width, output.height, request.video);
  let sx = f64::from(output.width) / f64::from(request.output.width);
  let sy = f64::from(output.height) / f64::from(request.output.height);
  output.crop_x *= sx;
  output.crop_y *= sy;
  output.crop_width *= sx;
  output.crop_height *= sy;
  output.image_x *= sx;
  output.image_y *= sy;
  output.image_width *= sx;
  output
}

/// `geometry`, placed for its own baked output, carried onto the canvas
/// `settings` describe.
pub(super) fn on_canvas(
  geometry: BakeGeometry,
  settings: &ScreenshotOutputSettings,
) -> BakeGeometry {
  let scale_x = f64::from(settings.width) / f64::from(geometry.output_width.max(1));
  let scale_y = f64::from(settings.height) / f64::from(geometry.output_height.max(1));
  let scaled = |value: u32, scale: f64| (f64::from(value) * scale).round() as u32;
  BakeGeometry {
    frame_x: (f64::from(geometry.frame_x) * scale_x).round() as i32,
    frame_y: (f64::from(geometry.frame_y) * scale_y).round() as i32,
    frame_width: scaled(geometry.frame_width, scale_x),
    frame_height: scaled(geometry.frame_height, scale_y),
    radius: scaled(geometry.radius, scale_x.min(scale_y)),
    output_width: settings.width,
    output_height: settings.height,
    ..geometry
  }
}
