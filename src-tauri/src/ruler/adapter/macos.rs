// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{osc::desktop::DesktopBinding, screenshots::CapturedImage};

use super::super::native_overlay_macos as native;

pub(crate) const fn available() -> bool {
  true
}

pub(crate) fn install(
  window: &tauri::WebviewWindow,
  anchor_id: u32,
  snapshots: &[(u32, CapturedImage)],
) -> Result<DesktopBinding, String> {
  native::install(window, anchor_id, snapshots)
}

pub(crate) fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
  native::show_without_activation(window)
}

pub(crate) fn present(window: &tauri::WebviewWindow) -> Result<(), String> {
  native::present(window)
}

pub(crate) fn set_screenshot_mode(
  window: &tauri::WebviewWindow,
  active: bool,
) -> Result<(), String> {
  native::set_screenshot_mode(window, active)
}

pub(crate) fn close(app: &tauri::AppHandle) {
  native::close(app);
}
