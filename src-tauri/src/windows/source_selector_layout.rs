// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

use super::WindowLabel;

pub(super) const SELECTOR_GAP: f64 = 6.0;

const SELECTOR_HEIGHT: f64 = 250.0;
const SELECTOR_WIDTH: f64 = 500.0;
const WINDOW_SELECTOR_EXPANDED_HEIGHT: f64 = 500.0;
const WINDOW_SELECTOR_EXPANDED_WIDTH: f64 = 750.0;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum SelectorPlacement {
  Above,
  Below,
}

pub(super) struct SelectorFrame {
  pub(super) position: LogicalPosition<f64>,
  pub(super) size: LogicalSize<f64>,
}

pub(super) fn selector_frame(
  app: &AppHandle,
  window_selector_active: bool,
) -> tauri::Result<(SelectorPlacement, SelectorFrame)> {
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  #[cfg(target_os = "windows")]
  let bar_position = bar.inner_position()?;
  #[cfg(not(target_os = "windows"))]
  let bar_position = bar.outer_position()?;
  #[cfg(target_os = "windows")]
  let bar_size = bar.inner_size()?;
  #[cfg(not(target_os = "windows"))]
  let bar_size = bar.outer_size()?;
  let monitor = bar
    .current_monitor()?
    .or(app.primary_monitor()?)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;

  let scale = monitor.scale_factor();
  let monitor_position = monitor.position().to_logical::<f64>(scale);
  let monitor_size = monitor.size().to_logical::<f64>(scale);
  let bar_position = bar_position.to_logical::<f64>(scale);
  let bar_size = bar_size.to_logical::<f64>(scale);
  #[cfg(target_os = "windows")]
  let selector_frame_offset = {
    let selector = app
      .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
      .ok_or_else(|| tauri::Error::WindowNotFound)?;
    let inner = selector.inner_position()?.to_logical::<f64>(scale);
    let outer = selector.outer_position()?.to_logical::<f64>(scale);
    LogicalPosition::new(inner.x - outer.x, inner.y - outer.y)
  };
  #[cfg(not(target_os = "windows"))]
  let selector_frame_offset = LogicalPosition::new(0.0, 0.0);
  let monitor_right = monitor_position.x + monitor_size.width;
  let bar_left = bar_position.x;
  let bar_top = bar_position.y;
  let bar_right = bar_left + bar_size.width;
  let bar_bottom = bar_top + bar_size.height;
  let (width, height) = selector_dimensions(window_selector_active, monitor_size.width);
  let gap = SELECTOR_GAP;
  let available_above = bar_top - monitor_position.y;
  let placement = if available_above >= height + gap {
    SelectorPlacement::Above
  } else {
    SelectorPlacement::Below
  };
  let center_x = (bar_left + bar_right) / 2.0;
  let x = (center_x - width / 2.0).clamp(monitor_position.x, monitor_right - width);
  let y = match placement {
    SelectorPlacement::Above => bar_top - gap - height,
    SelectorPlacement::Below => bar_bottom + gap,
  };

  Ok((
    placement,
    SelectorFrame {
      position: LogicalPosition::new(x - selector_frame_offset.x, y - selector_frame_offset.y),
      size: LogicalSize::new(width, height),
    },
  ))
}

pub(super) fn selector_dimensions(window_selector_active: bool, monitor_width: f64) -> (f64, f64) {
  let (width, height) = if window_selector_active {
    (
      WINDOW_SELECTOR_EXPANDED_WIDTH,
      WINDOW_SELECTOR_EXPANDED_HEIGHT,
    )
  } else {
    (SELECTOR_WIDTH, SELECTOR_HEIGHT)
  };

  (width.min(monitor_width), height)
}

#[cfg(test)]
mod tests {
  use super::selector_dimensions;

  #[test]
  fn monitor_selector_uses_its_content_size() {
    assert_eq!(selector_dimensions(false, 1_920.0), (500.0, 250.0));
  }

  #[test]
  fn selector_never_outgrows_the_monitor() {
    assert_eq!(selector_dimensions(true, 640.0), (640.0, 500.0));
  }
}
