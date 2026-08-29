// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

static CAPTURE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(target_os = "windows")]
fn prepare_windows(app: &AppHandle, restore_affinity: bool) -> tauri::Result<()> {
  let result = crate::windows::sync_capture_affinity(app, false);
  if result.is_err() {
    let _ = crate::windows::sync_capture_affinity(app, restore_affinity);
  }
  result
}

pub(crate) async fn capture_monitor_screenshot(
  app: AppHandle,
  monitor_id: u32,
) -> Result<crate::screenshots::CapturedImage, String> {
  // Monitor changes and adjacent magnifier requests may overlap. Capture
  // affinity is process-wide, so one capture must finish restoring it before
  // another begins its transaction.
  let _capture = CAPTURE.lock().await;
  #[cfg(not(target_os = "windows"))]
  let _ = &app;
  #[cfg(target_os = "windows")]
  let restore_affinity = crate::settings::current(&app).record_screenwide_windows;
  #[cfg(target_os = "windows")]
  {
    // Capture exclusion is a compositor state change. Applying it and taking
    // the frame in the same turn can still return the old, included overlay;
    // wait for DWM to present the excluded state without visually hiding any
    // Screenwide window.
    if let Err(error) = prepare_windows(&app, restore_affinity) {
      return Err(error.to_string());
    }
    tokio::time::sleep(std::time::Duration::from_millis(75)).await;
  }

  let screenshot = tauri::async_runtime::spawn_blocking(move || {
    #[cfg(target_os = "macos")]
    {
      crate::screenshots::capture_monitor_without_own_windows_blocking(monitor_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
      let monitor = xcap::Monitor::all()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|monitor| monitor.id().ok() == Some(monitor_id))
        .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
      monitor
        .capture_image()
        .map(|image| crate::screenshots::CapturedImage {
          width: image.width(),
          height: image.height(),
          rgba: image.into_raw(),
        })
        .map_err(|error| error.to_string())
    }
  })
  .await
  .map_err(|error| error.to_string());

  // Put the affinity back before the result is looked at, so a failed capture
  // cannot leave the windows excluded from the user's recordings.
  #[cfg(target_os = "windows")]
  let affinity_restore = crate::windows::sync_capture_affinity(&app, restore_affinity);
  #[cfg(target_os = "windows")]
  affinity_restore.map_err(|error| error.to_string())?;
  screenshot?
}
