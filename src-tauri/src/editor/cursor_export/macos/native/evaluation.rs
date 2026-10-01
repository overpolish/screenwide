// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::editor::cursor_export) fn scaled_size(
  request: &CursorExportRequest<'_>,
) -> Result<(u32, u32), String> {
  let placement =
    crate::screenshots::output_placement(request.width, request.height, request.output)?;
  Ok((placement.image_width, placement.image_height))
}

/// Evaluates one cursor per 60 Hz timeline frame in output pixels. The cursor
/// is evaluated in the placed image's pixel space (so artwork size, hotspot
/// and motion-blur delta scale exactly as the retired pre-pass scaled them)
/// and then offset onto the canvas by the image origin.
pub(in crate::editor::cursor_export) fn evaluate(
  request: &CursorExportRequest<'_>,
) -> Result<Option<CursorTimeline>, String> {
  let Some(cursor_path) = request.cursor else {
    return Ok(None);
  };
  let cursor = CursorCompositor::open(cursor_path)?;
  let (output_width, output_height) = scaled_size(request)?;
  let placement =
    crate::screenshots::output_placement(request.width, request.height, request.output)?;
  let frame_count = request
    .duration_ms
    .saturating_mul(CURSOR_FRAME_RATE)
    .div_ceil(1_000)
    .saturating_add(1);
  let frames = (0..frame_count)
    .map(|frame| {
      let position_ms = frame.saturating_mul(1_000) / CURSOR_FRAME_RATE;
      cursor
        .gpu_cursor(
          position_ms,
          (output_width, output_height),
          request.cursor_effects,
        )
        .map(|mut cursor| {
          cursor.x += placement.image_x as f32;
          cursor.y += placement.image_y as f32;
          cursor
        })
    })
    .collect();
  Ok(Some(CursorTimeline {
    artworks: crate::editor::cursor_effects::gpu_artworks(),
    frames,
  }))
}

pub(in crate::editor::cursor_export) fn evaluate_keyboard(
  request: &CursorExportRequest<'_>,
) -> Result<Option<KeyboardTimeline>, String> {
  let Some(keyboard_path) = request.keyboard else {
    return Ok(None);
  };
  let deleted_keyboard_shortcut_ids = request
    .timeline
    .map_or(&[][..], |timeline| timeline.deleted_keyboard_shortcut_ids());
  let deleted_keyboard_shortcut_ranges = request.timeline.map_or(&[][..], |timeline| {
    timeline.deleted_keyboard_shortcut_ranges()
  });
  let keyboard = KeyboardCompositor::open_with_deleted(
    keyboard_path,
    deleted_keyboard_shortcut_ids,
    deleted_keyboard_shortcut_ranges,
  )?;
  if let Some(timeline) = request.timeline {
    keyboard.set_shortcut_positions(timeline.keyboard_shortcut_positions());
  }
  let dimensions = (request.output.width, request.output.height);
  let frame_count = request
    .duration_ms
    .saturating_mul(CURSOR_FRAME_RATE)
    .div_ceil(1_000)
    .saturating_add(1);
  let frames = (0..frame_count)
    .map(|frame| {
      let position_ms = frame.saturating_mul(1_000) / CURSOR_FRAME_RATE;
      keyboard
        .evaluate_fitted_with_timeline(
          position_ms,
          request.keyboard_effects,
          dimensions,
          request.timeline,
        )
        .unwrap_or_default()
    })
    .collect();
  Ok(Some(KeyboardTimeline { frames }))
}
