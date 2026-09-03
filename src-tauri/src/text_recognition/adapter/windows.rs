// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::screenshots::CapturedImage;

use super::super::native_overlay_windows as native;

pub(crate) fn install(
  window: &tauri::WebviewWindow,
  anchor_id: u32,
  snapshots: &[(u32, CapturedImage)],
) -> Result<bool, String> {
  // A compositor that could not be created is not an error: the webview
  // implementation stays the safety net.
  match native::install(window, anchor_id, snapshots) {
    Ok(()) => Ok(true),
    Err(error) => {
      eprintln!("The Windows text recognition overlay fell back to the webview: {error}");
      Ok(false)
    }
  }
}

pub(crate) fn show_interactive(window: &tauri::WebviewWindow) -> Result<(), String> {
  native::show_interactive(window)
}

pub(crate) fn present(window: &tauri::WebviewWindow) -> Result<(), String> {
  native::present(window)
}

pub(crate) fn close(app: &tauri::AppHandle, except: Option<&str>) {
  native::close(app, except);
}

pub(crate) fn render(app: &tauri::AppHandle, packet: super::super::visual::RenderPacket) {
  native::render(app, packet);
}

pub(crate) fn render_window(
  window: &tauri::WebviewWindow,
  packet: super::super::visual::RenderPacket,
) {
  native::render_window(window, packet);
}
