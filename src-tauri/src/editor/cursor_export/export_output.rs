// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn scaled(request: &CursorExportRequest<'_>) -> ScreenshotOutputSettings {
  let mut output = request.output.clone();
  (output.width, output.height) =
    super::super::output_dimensions(output.width, output.height, request.video);
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
