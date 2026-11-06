// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Launch at login, through the autostart plugin, or through the startup
//! task of the Microsoft Store package when the app runs from it.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn is_enabled(app: &AppHandle) -> bool {
  #[cfg(target_os = "windows")]
  if crate::microsoft_store::is_packaged() {
    return crate::microsoft_store::startup_task::is_enabled();
  }
  app.autolaunch().is_enabled().unwrap_or(false)
}

/// Turns launch at login on or off and answers whether it is now on.
pub fn set(app: &AppHandle, enabled: bool) -> Result<bool, String> {
  #[cfg(target_os = "windows")]
  if crate::microsoft_store::is_packaged() {
    return crate::microsoft_store::startup_task::set(enabled);
  }
  let autolaunch = app.autolaunch();
  if enabled {
    autolaunch.enable()
  } else {
    autolaunch.disable()
  }
  .map_err(|error| error.to_string())?;
  Ok(autolaunch.is_enabled().unwrap_or(enabled))
}
