// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

use super::EditorKind;
use crate::windows;

pub fn show(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  let window = app
    .get_webview_window(kind.window_label().as_str())
    .ok_or(tauri::Error::WindowNotFound)?;

  #[cfg(target_os = "macos")]
  app.set_dock_visibility(true)?;

  windows::show(&window, true)?;
  let _ = windows::recover_window_position(app, &window);

  Ok(())
}

pub fn hide(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  // An open options window is a child of this one: left attached, it would
  // come back with the editor the next time it is shown.
  let _ = super::export_window::hide(app, kind);
  // A tool panel hangs off this window and stays up through an outside press,
  // so it has to be taken away with the editor it belongs to.
  windows::options::close_standalone_listbox_for_parent(app, kind.window_label().as_str());
  if let Some(window) = app.get_webview_window(kind.window_label().as_str()) {
    windows::hide_without_focus_transfer(&window)?;
  }

  Ok(())
}
