// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, LogicalPosition, PhysicalPosition, PhysicalSize, WebviewWindow};

/// Where a window's page begins on screen, in logical pixels: the origin that
/// page coordinates, such as a control's bounds, are measured from. The page
/// says how tall it is (`window.innerHeight`). On macOS a window with a title
/// bar of its own, as the Storybook preview has, reports its content as the
/// whole window while the page begins below that bar, so the page starts as
/// far down as the content is taller than the page. A page that fills its
/// window, as every app window's does, starts at the content's corner.
pub(crate) fn page_origin(
  window: &WebviewWindow,
  page_height: Option<f64>,
) -> tauri::Result<LogicalPosition<f64>> {
  let scale = window.scale_factor()?;
  let origin = window.inner_position()?.to_logical::<f64>(scale);
  let inset = match page_height {
    Some(height) => (window.inner_size()?.to_logical::<f64>(scale).height - height).max(0.0),
    None => 0.0,
  };
  Ok(LogicalPosition::new(origin.x, origin.y + inset))
}

/// The area, in physical pixels, that a window shares with a monitor.
pub(super) fn overlap_area(
  monitor_position: PhysicalPosition<i32>,
  monitor_size: PhysicalSize<u32>,
  window_position: PhysicalPosition<i32>,
  window_size: PhysicalSize<u32>,
) -> i64 {
  let left = window_position.x.max(monitor_position.x);
  let top = window_position.y.max(monitor_position.y);
  let right = (window_position.x + window_size.width as i32)
    .min(monitor_position.x + monitor_size.width as i32);
  let bottom = (window_position.y + window_size.height as i32)
    .min(monitor_position.y + monitor_size.height as i32);

  i64::from((right - left).max(0)) * i64::from((bottom - top).max(0))
}

/// Whether any part of a window still lands on a connected monitor. A saved
/// position stops being usable the moment its display is unplugged or moved.
pub(super) fn window_is_on_a_monitor(
  app: &AppHandle,
  window: &WebviewWindow,
) -> tauri::Result<bool> {
  let window_position = window.outer_position()?;
  let window_size = window.outer_size()?;

  Ok(app.available_monitors()?.iter().any(|monitor| {
    overlap_area(
      *monitor.position(),
      *monitor.size(),
      window_position,
      window_size,
    ) > 0
  }))
}

pub(super) fn keep_window_on_a_monitor(
  app: &AppHandle,
  window: &WebviewWindow,
) -> tauri::Result<()> {
  let window_size = window.outer_size()?;

  if !window_is_on_a_monitor(app, window)? {
    if let Some(monitor) = app.primary_monitor()? {
      let position = monitor.position();
      let size = monitor.size();
      window.set_position(PhysicalPosition {
        x: position.x + (size.width.saturating_sub(window_size.width) / 2) as i32,
        y: position.y + size.height.saturating_sub(window_size.height + 100) as i32,
      })?;
    }
  }

  Ok(())
}
