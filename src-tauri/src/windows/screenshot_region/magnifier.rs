// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn prepare_screenshot_region_magnifier(
  app: AppHandle,
  window: String,
  monitor_id: u32,
) -> Result<bool, String> {
  let screenshot =
    super::super::monitor_capture::capture_monitor_screenshot(app.clone(), monitor_id).await?;
  let target = app
    .get_webview_window(&window)
    .ok_or_else(|| format!("Screenshot overlay not found: {window}"))?;
  super::adapter::set_magnifier_source(&target, screenshot)
}
