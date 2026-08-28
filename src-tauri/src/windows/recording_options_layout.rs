// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Measurement, placement, and anchor hit-testing for recording options.

use std::sync::{Mutex, MutexGuard};

use serde::Deserialize;
use tauri::{AppHandle, LogicalPosition, Manager};

use super::WindowLabel;

pub const WIDTH: f64 = 240.0;
const FALLBACK_HEIGHT: f64 = 323.0;
const GAP: f64 = 6.0;

static LAYOUT: Mutex<Layout> = Mutex::new(Layout {
  anchor: PopoverAnchor::EMPTY,
  height: FALLBACK_HEIGHT,
});

#[derive(Clone, Copy)]
pub struct Layout {
  pub anchor: PopoverAnchor,
  pub height: f64,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopoverAnchor {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

impl PopoverAnchor {
  const EMPTY: Self = Self {
    x: 0.0,
    y: 0.0,
    width: 0.0,
    height: 0.0,
  };

  fn center_x(self) -> f64 {
    self.x + self.width / 2.0
  }
}

fn layout() -> MutexGuard<'static, Layout> {
  LAYOUT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn set_anchor(anchor: PopoverAnchor) -> Layout {
  let mut layout = layout();
  layout.anchor = anchor;
  *layout
}

pub fn set_height(height: f64) -> Option<Layout> {
  let mut layout = layout();
  if (layout.height - height).abs() < 0.5 {
    return None;
  }
  layout.height = height;
  Some(*layout)
}

pub fn frame(app: &AppHandle, layout: Layout) -> tauri::Result<LogicalPosition<f64>> {
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let monitor = bar
    .current_monitor()?
    .or(app.primary_monitor()?)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let scale = monitor.scale_factor();
  let monitor_position = monitor.position().to_logical::<f64>(scale);
  let monitor_size = monitor.size().to_logical::<f64>(scale);
  let bar_position = bar.outer_position()?.to_logical::<f64>(scale);
  let bar_size = bar.outer_size()?.to_logical::<f64>(scale);
  let monitor_right = monitor_position.x + monitor_size.width;
  let monitor_bottom = monitor_position.y + monitor_size.height;
  let x = (bar_position.x + layout.anchor.center_x() - WIDTH / 2.0)
    .clamp(monitor_position.x, monitor_right - WIDTH);
  let available_above = bar_position.y - monitor_position.y;
  let y = if available_above >= layout.height + GAP {
    bar_position.y - layout.height - GAP
  } else {
    (bar_position.y + bar_size.height + GAP).min(monitor_bottom - layout.height)
  };

  Ok(LogicalPosition::new(x, y))
}

pub fn anchor_contains(app: &AppHandle, x: f64, y: f64) -> bool {
  let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) else {
    return false;
  };
  let Ok(position) = bar.outer_position() else {
    return false;
  };
  let Ok(scale) = bar.scale_factor() else {
    return false;
  };
  let position = position.to_logical::<f64>(scale);
  let anchor = layout().anchor;
  let left = position.x + anchor.x;
  let top = position.y + anchor.y;

  x >= left && x <= left + anchor.width && y >= top && y <= top + anchor.height
}
