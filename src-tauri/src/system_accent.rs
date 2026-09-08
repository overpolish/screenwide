// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "macos")]
#[path = "system_accent/platform_macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "system_accent/platform_windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "system_accent/platform_other.rs"]
mod platform;

/// Emitted to every window whenever the operating system accent changes. The
/// payload is the new accent, or `null` when the app should use its own.
const ACCENT_CHANGED_EVENT: &str = "system-accent-changed";

/// The operating system accent colour, in sRGB.
#[derive(Clone, Copy, Serialize)]
pub struct SystemAccent {
  red: u8,
  green: u8,
  blue: u8,
}

/// Reads the current accent. `None` means the platform has no accent to
/// follow, so the app keeps its own brand colour.
#[tauri::command]
pub fn get_system_accent() -> Option<SystemAccent> {
  platform::current()
}

/// Installs the platform observer once, at startup.
pub fn initialize(app: &AppHandle) {
  platform::initialize(app, accent_changed);
}

fn accent_changed(app: AppHandle) {
  let _ = app.emit(ACCENT_CHANGED_EVENT, platform::current());
}
