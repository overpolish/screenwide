// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{osc::desktop::DesktopBinding, screenshots::CapturedImage};

pub(crate) const fn available() -> bool {
  false
}

pub(crate) fn install(
  _window: &tauri::WebviewWindow,
  _anchor_id: u32,
  _snapshots: &[(u32, CapturedImage)],
) -> Result<DesktopBinding, String> {
  Err("The native Ruler compositor is not available on this platform".to_owned())
}

pub(crate) fn show_interactive(_window: &tauri::WebviewWindow) -> Result<(), String> {
  Ok(())
}

pub(crate) fn present(_window: &tauri::WebviewWindow) -> Result<(), String> {
  Ok(())
}

pub(crate) fn set_screenshot_mode(
  _window: &tauri::WebviewWindow,
  _active: bool,
) -> Result<(), String> {
  Ok(())
}

pub(crate) fn close(app: &tauri::AppHandle) {
  for window in super::super::ruler_windows(app) {
    let _ = window.close();
  }
}
