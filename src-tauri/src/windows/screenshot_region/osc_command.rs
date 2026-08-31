// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use super::adapter::{self, RegionSceneRequest};
use crate::osc::geometry::{Point, Rect, Size};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ExclusionRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

/// Synchronizes frontend workflow state into the native region OSC.
#[tauri::command]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri exposes this function as a flat, named IPC command"
)]
pub fn set_screenshot_region_osc(
  app: AppHandle,
  window: String,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  visible: bool,
  aspect: Option<f64>,
  input_enabled: bool,
  exclusion_rect: Option<ExclusionRect>,
  show_frame: bool,
  show_handles: bool,
  allow_drawing: bool,
  monitor_width: f64,
  monitor_height: f64,
  desktop: bool,
  monitor_id: Option<u32>,
) -> Result<bool, String> {
  let target = app
    .get_webview_window(&window)
    .ok_or_else(|| format!("Screenshot overlay not found: {window}"))?;
  let desktop_anchor = desktop
    .then(|| monitor_id.ok_or_else(|| "Desktop Region OSC requires a monitor".to_owned()))
    .transpose()?;
  let rect = Rect {
    origin: Point { x, y },
    size: Size { width, height },
  };
  let exclusion_rect = exclusion_rect.map(|rect| Rect {
    origin: Point {
      x: rect.x,
      y: rect.y,
    },
    size: Size {
      width: rect.width,
      height: rect.height,
    },
  });
  adapter::apply_region_scene(
    &app,
    target,
    RegionSceneRequest {
      rect,
      visible,
      aspect,
      input_enabled,
      exclusion_rect,
      show_frame,
      show_handles,
      allow_drawing,
      monitor_width,
      monitor_height,
      desktop_anchor,
    },
  )
}
