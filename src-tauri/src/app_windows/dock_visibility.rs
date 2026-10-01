// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::WindowLabel;

fn has_visible_workspace(app: &AppHandle) -> bool {
  [
    WindowLabel::EditorRecording,
    WindowLabel::EditorScreenshot,
    WindowLabel::ExportRecording,
    WindowLabel::ExportScreenshot,
    WindowLabel::Settings,
    WindowLabel::Update,
  ]
  .iter()
  .filter_map(|label| app.get_webview_window(label.as_str()))
  .any(|window| window.is_visible().unwrap_or(true))
}

/// Called on the main thread after native dismissal has actually hidden a window.
pub fn sync_after_dismissal(app: &AppHandle) -> tauri::Result<()> {
  if has_visible_workspace(app) {
    return Ok(());
  }

  app.set_dock_visibility(false)?;
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    // Tao ignores hides within one second of showing the Dock icon. Retry
    // beyond that guard for windows opened and closed in quick succession.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
      // A workspace may have reopened while waiting. Query and hide together
      // on the main thread so a new show cannot interleave with this decision.
      if !has_visible_workspace(&handle) {
        let _ = handle.set_dock_visibility(false);
      }
    });
  });
  Ok(())
}
