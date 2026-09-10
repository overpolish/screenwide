// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{editor, windows};
use tauri::{AppHandle, Manager};

pub(crate) fn hide_unrequested_windows(app: &AppHandle) -> tauri::Result<()> {
  // Native effects can order ordinary app windows during setup. Their
  // first presentation always belongs to an explicit user action.
  let app_handle = app.clone();
  // Recovery may have put an artifact in one workspace; that window is
  // the one presentation the user did ask for, so it alone is spared.
  let mut labels = vec![windows::WindowLabel::Settings];
  labels.extend(editor::export_window::LABELS);
  labels.extend(
    editor::EditorKind::ALL
      .into_iter()
      .filter(|kind| !editor::has_pending_workspace_kind(app, *kind))
      .map(editor::EditorKind::window_label),
  );
  app.run_on_main_thread(move || {
    for label in &labels {
      if let Some(window) = app_handle.get_webview_window(label.as_str()) {
        let _ = windows::hide(&window);
      }
    }
  })?;
  Ok(())
}
