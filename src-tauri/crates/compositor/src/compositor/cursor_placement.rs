// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the recorded pointer lands on a canvas. The compositor draws the
//! cursor in canvas pixels, so whoever evaluates it places it first.

use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings, GpuCursor};
use screenwide_output::{output_placement, OutputPlacement, ScreenshotOutputSettings};

/// `cursor` at `position_ms` on the canvas `output` lays a `source`-sized
/// picture on. It is evaluated in the placed picture's own pixels, so its size,
/// hotspot and blur scale with the picture, then moved by the picture's origin.
pub fn placed_cursor(
  cursor: &CursorCompositor,
  position_ms: u64,
  settings: CursorEffectSettings,
  source: (u32, u32),
  output: &ScreenshotOutputSettings,
) -> Result<Option<GpuCursor>, String> {
  let placement = output_placement(source.0, source.1, output)?;
  Ok(
    cursor
      .gpu_cursor(
        position_ms,
        (placement.image_width, placement.image_height),
        settings,
      )
      .map(|mut cursor| {
        cursor.x += placement.image_x as f32;
        cursor.y += placement.image_y as f32;
        cursor
      }),
  )
}

/// `cursor`, placed where `from` put the picture, carried to where `to` puts
/// it: an edit or a scene moved and scaled the picture under it.
pub fn carried_cursor(
  mut cursor: GpuCursor,
  from: &OutputPlacement,
  to: &OutputPlacement,
) -> GpuCursor {
  if from.image_width == 0 || from.image_height == 0 {
    return cursor;
  }
  let sx = f64::from(to.image_width) / f64::from(from.image_width);
  let sy = f64::from(to.image_height) / f64::from(from.image_height);
  cursor.x = (to.image_x + (f64::from(cursor.x) - from.image_x) * sx) as f32;
  cursor.y = (to.image_y + (f64::from(cursor.y) - from.image_y) * sy) as f32;
  cursor.width *= sx as f32;
  cursor.height *= sy as f32;
  cursor.hotspot_x *= sx as f32;
  cursor.hotspot_y *= sy as f32;
  cursor.blur_delta_x *= sx as f32;
  cursor.blur_delta_y *= sy as f32;
  cursor
}
