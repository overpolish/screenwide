// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Carrying what the export placed once onto the canvas a frame is drawn on.

use screenwide_cursor_effects::GpuCursor;
use screenwide_output::{output_placement, ScreenshotOutputSettings};

/// `cursor`, evaluated on the canvas where `from` places a `source`-sized
/// screen, carried to where `to` places it. The export evaluates its cursors
/// once, on the request's own composition, and a scene then moves and scales
/// the screen under them frame by frame, a zoom further than its box.
pub(super) fn carried_cursor(
  mut cursor: GpuCursor,
  source: (u32, u32),
  from: &ScreenshotOutputSettings,
  to: &ScreenshotOutputSettings,
) -> Result<GpuCursor, String> {
  let from = output_placement(source.0, source.1, from)?;
  let to = output_placement(source.0, source.1, to)?;
  let scale = f64::from(to.image_width) / f64::from(from.image_width.max(1));
  cursor.x = (to.image_x + (f64::from(cursor.x) - from.image_x) * scale) as f32;
  cursor.y = (to.image_y + (f64::from(cursor.y) - from.image_y) * scale) as f32;
  let scale = scale as f32;
  cursor.width *= scale;
  cursor.height *= scale;
  cursor.hotspot_x *= scale;
  cursor.hotspot_y *= scale;
  cursor.blur_delta_x *= scale;
  cursor.blur_delta_y *= scale;
  Ok(cursor)
}
