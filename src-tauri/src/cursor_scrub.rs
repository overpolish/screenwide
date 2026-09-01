// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use tauri::ipc::Channel;

#[cfg(target_os = "macos")]
#[path = "cursor_scrub/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "cursor_scrub/windows.rs"]
mod platform;

#[derive(Clone, Serialize)]
#[cfg_attr(target_os = "macos", allow(dead_code))]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub(super) enum CursorScrubEvent {
  Move {
    alt_key: bool,
    delta_x: i32,
    delta_y: i32,
    shift_key: bool,
  },
  End,
}

#[tauri::command]
pub fn begin_cursor_scrub(channel: Channel<CursorScrubEvent>) -> Result<(), String> {
  platform::begin(channel)
}

#[tauri::command]
pub fn end_cursor_scrub(
  window: tauri::WebviewWindow,
  cursor_offset_x: Option<f64>,
) -> Result<(), String> {
  let offset = cursor_offset_x.unwrap_or(0.0);
  #[cfg(target_os = "windows")]
  let offset = offset * window.scale_factor().map_err(|error| error.to_string())?;
  #[cfg(target_os = "macos")]
  let _ = window;
  platform::end(offset)
}

#[cfg(target_os = "macos")]
pub(crate) fn pin_cursor_at(point: core_graphics::geometry::CGPoint) -> Result<(), String> {
  platform::pin_cursor_at(point)
}

#[cfg(target_os = "macos")]
pub(crate) fn restore_cursor_at(point: core_graphics::geometry::CGPoint) {
  platform::restore_cursor_at(point);
}

#[cfg(test)]
mod tests {
  use super::CursorScrubEvent;

  #[test]
  fn movement_payload_uses_frontend_field_names() {
    let payload = serde_json::to_value(CursorScrubEvent::Move {
      alt_key: true,
      delta_x: 3,
      delta_y: -2,
      shift_key: false,
    })
    .unwrap();

    assert_eq!(
      payload,
      serde_json::json!({
        "altKey": true,
        "deltaX": 3,
        "deltaY": -2,
        "shiftKey": false,
        "type": "move",
      })
    );
  }
}
