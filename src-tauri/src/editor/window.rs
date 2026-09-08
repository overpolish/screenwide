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
  let _ = windows::contain_editor(app, &window);

  Ok(())
}

pub fn hide(app: &AppHandle, kind: EditorKind) -> tauri::Result<()> {
  // An open options window is a child of this one: left attached, it would
  // come back with the editor the next time it is shown.
  let _ = super::export_window::hide(app, kind);
  if let Some(window) = app.get_webview_window(kind.window_label().as_str()) {
    windows::hide_without_focus_transfer(&window)?;
  }

  Ok(())
}
