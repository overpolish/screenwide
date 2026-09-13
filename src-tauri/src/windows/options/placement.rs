// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving an open panel with the window it hangs off.

use tauri::{AppHandle, LogicalPosition, Manager};

use super::{context_for, panel_label, STANDALONE_LISTBOX};

/// Re-places an open panel against its parent's content without touching its
/// attachment, level or visibility.
///
/// A sticky panel is placed against the preview rather than the window frame,
/// so an editor resize moves the corner it hangs from. Re-showing it would
/// re-attach and re-order the window for a change that is only a position.
// Keep this command synchronous so Tauri runs it on the main thread, like
// show/hide. An async worker can hold STANDALONE_LISTBOX while a window query
// waits for the main thread, which may itself be opening a panel and waiting
// for that same lock.
#[tauri::command]
pub fn move_standalone_listbox(
  app: AppHandle,
  parent_window_label: String,
  offset: LogicalPosition<f64>,
  panel: Option<String>,
) -> tauri::Result<()> {
  let panel = panel_label(panel);
  let _lifecycle = STANDALONE_LISTBOX.lock();
  let belongs_to_parent =
    context_for(&panel).is_some_and(|context| context.parent_window_label == parent_window_label);
  if !belongs_to_parent {
    return Ok(());
  }
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or(tauri::Error::WindowNotFound)?;
  let window = app
    .get_webview_window(&panel)
    .ok_or(tauri::Error::WindowNotFound)?;
  #[cfg(target_os = "windows")]
  {
    super::placement_windows::place(&app, &parent, &window, offset, None)?;
    if let Some(context) = super::standalone_listbox_contexts().get_mut(&panel) {
      context.offset = offset;
    }
    Ok(())
  }
  #[cfg(not(target_os = "windows"))]
  {
    let scale = parent.scale_factor()?;
    let parent_position = parent.outer_position()?.to_logical::<f64>(scale);
    let size = window
      .outer_size()?
      .to_logical::<f64>(window.scale_factor()?);
    let mut position =
      LogicalPosition::new(parent_position.x + offset.x, parent_position.y + offset.y);
    if let Some(monitor) = parent.current_monitor()?.or(app.primary_monitor()?) {
      let monitor_scale = monitor.scale_factor();
      let monitor_position = monitor.position().to_logical::<f64>(monitor_scale);
      let monitor_size = monitor.size().to_logical::<f64>(monitor_scale);
      let max_x = monitor_position.x + (monitor_size.width - size.width).max(0.0);
      let max_y = monitor_position.y + (monitor_size.height - size.height).max(0.0);
      position.x = position.x.clamp(monitor_position.x, max_x);
      position.y = position.y.clamp(monitor_position.y, max_y);
    }
    window.set_position(position)
  }
}
