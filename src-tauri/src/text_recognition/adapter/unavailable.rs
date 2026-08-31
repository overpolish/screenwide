// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::screenshots::CapturedImage;

pub(crate) fn install(
  _window: &tauri::WebviewWindow,
  _anchor_id: u32,
  _snapshots: &[(u32, CapturedImage)],
) -> Result<bool, String> {
  Ok(false)
}

pub(crate) fn show_without_activation(_window: &tauri::WebviewWindow) -> Result<(), String> {
  Ok(())
}

pub(crate) fn present(_window: &tauri::WebviewWindow) -> Result<(), String> {
  Ok(())
}

pub(crate) fn close(app: &tauri::AppHandle, except: Option<&str>) {
  for window in super::super::recognition_windows(app) {
    if Some(window.label()) != except {
      #[cfg(target_os = "windows")]
      let _ = crate::windows::conceal_disposable_overlay(&window);
      let _ = window.close();
    }
  }
}

pub(crate) fn render(_app: &tauri::AppHandle, _packet: super::super::visual::RenderPacket) {}

pub(crate) fn render_window(
  _window: &tauri::WebviewWindow,
  _packet: super::super::visual::RenderPacket,
) {
}
