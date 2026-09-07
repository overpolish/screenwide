// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Monitor};

/// One monitor paired across the capture and native window APIs.
pub(crate) struct Display {
  pub id: u32,
  pub capture: xcap::Monitor,
  pub native: Monitor,
  pub layout_position: (f64, f64),
  pub layout_size: (f64, f64),
}

/// Reuses the selector's cross-API mapping: Tauri has no capture identifier,
/// so pair by enumeration order and reject mismatched counts.
pub(crate) fn snapshot(app: &AppHandle) -> Result<Vec<Display>, String> {
  let capture_monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
  let native_monitors = app
    .available_monitors()
    .map_err(|error| error.to_string())?;
  if capture_monitors.len() != native_monitors.len() {
    return Err("Tauri and xcap returned different monitor counts".into());
  }

  capture_monitors
    .into_iter()
    .zip(native_monitors)
    .map(|(capture, native)| {
      let id = capture.id().map_err(|error| error.to_string())?;
      let layout_position = (
        capture.x().map_err(|error| error.to_string())? as f64,
        capture.y().map_err(|error| error.to_string())? as f64,
      );
      let layout_size = (
        capture.width().map_err(|error| error.to_string())? as f64,
        capture.height().map_err(|error| error.to_string())? as f64,
      );
      Ok(Display {
        id,
        capture,
        native,
        layout_position,
        layout_size,
      })
    })
    .collect()
}
