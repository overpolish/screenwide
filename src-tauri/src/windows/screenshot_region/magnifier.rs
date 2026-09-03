// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn prepare_screenshot_region_magnifier(
  app: AppHandle,
  window: String,
  monitor_id: u32,
) -> Result<bool, String> {
  let target = app
    .get_webview_window(&window)
    .ok_or_else(|| format!("Screenshot overlay not found: {window}"))?;
  // A resize may cross a display boundary while the anchor stays fixed. Keep
  // one source per compositor surface so the lens can follow the active edge
  // and sample the display it is actually over.
  let mut monitor_ids = xcap::Monitor::all()
    .map_err(|error| error.to_string())?
    .into_iter()
    .filter_map(|monitor| monitor.id().ok())
    .collect::<Vec<_>>();
  monitor_ids.sort_unstable_by_key(|id| u8::from(*id != monitor_id));

  let mut anchor_ready = false;
  for display_id in monitor_ids {
    let screenshot = match super::super::monitor_capture::capture_monitor_screenshot(
      app.clone(),
      display_id,
    )
    .await
    {
      Ok(screenshot) => screenshot,
      Err(error) if display_id == monitor_id => return Err(error),
      Err(_) => continue,
    };
    let ready = super::adapter::set_magnifier_source(&target, display_id, screenshot)?;
    if display_id == monitor_id {
      anchor_ready = ready;
    }
  }
  Ok(anchor_ready)
}
