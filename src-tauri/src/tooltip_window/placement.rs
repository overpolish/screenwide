// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a tooltip stands against the control it describes.

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

use super::{AnchorRect, GAP};

/// Where a tooltip of `size` belongs: centred under its anchor, flipped above
/// it when there is no room below, and never off the edge of the display the
/// anchor is on. Computed from the requested size, because macOS has queued
/// the resize rather than applied it.
pub(super) fn placement(
  app: &AppHandle,
  parent_label: &str,
  anchor: AnchorRect,
  size: LogicalSize<f64>,
) -> LogicalPosition<f64> {
  let mut position = LogicalPosition::new(
    anchor.x + (anchor.width - size.width) / 2.0,
    anchor.y + anchor.height + GAP,
  );

  let Some(monitor) = anchor_monitor(app, parent_label, anchor) else {
    return position;
  };
  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  let origin = work_area.position.to_logical::<f64>(scale);
  let extent = work_area.size.to_logical::<f64>(scale);

  if position.y + size.height > origin.y + extent.height {
    position.y = anchor.y - size.height - GAP;
  }
  let max_x = origin.x + (extent.width - size.width).max(0.0);
  let max_y = origin.y + (extent.height - size.height).max(0.0);
  position.x = position.x.clamp(origin.x, max_x.max(origin.x));
  position.y = position.y.clamp(origin.y, max_y.max(origin.y));
  position
}

/// The display the described control is on, which is the one the tooltip has
/// to stay inside. Asked for by point rather than taken from the tooltip
/// window, which is still wherever the last tooltip left it.
fn anchor_monitor(
  app: &AppHandle,
  parent_label: &str,
  anchor: AnchorRect,
) -> Option<tauri::Monitor> {
  let scale = app
    .get_webview_window(parent_label)
    .and_then(|parent| parent.scale_factor().ok())
    .unwrap_or(1.0);
  let center = (
    (anchor.x + anchor.width / 2.0) * scale,
    (anchor.y + anchor.height / 2.0) * scale,
  );
  match app.monitor_from_point(center.0, center.1) {
    Ok(Some(monitor)) => Some(monitor),
    _ => app.primary_monitor().ok().flatten(),
  }
}
