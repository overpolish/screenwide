// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{
  AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
};

use super::{placement_geometry::clamped_axis, standalone_listbox_contexts};

pub(super) fn place(
  app: &AppHandle,
  parent: &WebviewWindow,
  panel: &WebviewWindow,
  offset: LogicalPosition<f64>,
  size: Option<LogicalSize<f64>>,
) -> tauri::Result<()> {
  let scale = parent.scale_factor()?;
  let origin = parent.inner_position()?;
  let size = match size {
    Some(size) => size,
    None => panel.inner_size()?.to_logical::<f64>(panel.scale_factor()?),
  };
  let size: PhysicalSize<u32> = size.to_physical(scale);
  let mut position = PhysicalPosition::new(
    (f64::from(origin.x) + offset.x * scale).round() as i32,
    (f64::from(origin.y) + offset.y * scale).round() as i32,
  );
  if let Some(monitor) = parent.current_monitor()?.or(app.primary_monitor()?) {
    let area = monitor.work_area();
    position.x = clamped_axis(
      origin.x,
      offset.x,
      scale,
      size.width,
      area.position.x,
      area.size.width,
    );
    position.y = clamped_axis(
      origin.y,
      offset.y,
      scale,
      size.height,
      area.position.y,
      area.size.height,
    );
  }
  panel.set_position(position)?;
  panel.set_size(size)
}

/// Win32 ownership handles z-order/minimization, but does not move an owned
/// window when its owner moves. Keep the UI's preview-relative anchor here.
pub(crate) fn follow_parent(app: &AppHandle, parent: &WebviewWindow) {
  if parent.is_minimized().unwrap_or(false) {
    return;
  }
  let panels = standalone_listbox_contexts()
    .iter()
    .filter(|(_, context)| context.open && context.attached_to.as_deref() == Some(parent.label()))
    .map(|(label, context)| (label.clone(), context.offset))
    .collect::<Vec<_>>();
  for (label, offset) in panels {
    if let Some(panel) = app.get_webview_window(&label) {
      if let Err(error) = place(app, parent, &panel, offset, None) {
        eprintln!("Could not move the editor tool panel: {error}");
      }
    }
  }
}
