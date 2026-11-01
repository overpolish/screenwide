// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

use super::EditorKind;
use crate::app_windows;

pub fn show(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let window = app
    .get_webview_window(kind.window_label().as_str())
    .ok_or(tauri::Error::WindowNotFound)?;

  #[cfg(target_os = "macos")]
  app.set_dock_visibility(true)?;

  app_windows::show(&window, true)?;
  let _ = app_windows::recover_window_position(app, &window);

  Ok(())
}

pub fn hide(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  // Nothing measures the export's size for a window that is away.
  if kind == EditorKind::Recording {
    super::preview::cancel_export_work(&app.state::<super::EditorState>());
  }
  // A tool panel hangs off this window and stays up through an outside press,
  // so it has to be taken away with the editor it belongs to.
  app_windows::options::close_standalone_listbox_for_parent(app, kind.window_label().as_str());
  if let Some(window) = app.get_webview_window(kind.window_label().as_str()) {
    app_windows::hide_without_focus_transfer(&window)?;
  }

  Ok(())
}
