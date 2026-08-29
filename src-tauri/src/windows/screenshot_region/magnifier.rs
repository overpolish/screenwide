// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;

#[cfg(target_os = "macos")]
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
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = target
        .ns_view()
        .map(|view| {
          super::native_osc_macos::set_magnifier_source(
            view.cast(),
            &screenshot.rgba,
            screenshot.width,
            screenshot.height,
          )
        })
        .map_err(|error| error.to_string());
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn prepare_screenshot_region_magnifier(
  _app: AppHandle,
  _window: String,
  _monitor_id: u32,
) -> Result<bool, String> {
  Ok(false)
}
