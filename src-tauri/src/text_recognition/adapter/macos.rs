// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::screenshots::CapturedImage;

use super::super::native_overlay_macos as native;

pub(crate) fn install(
  window: &tauri::WebviewWindow,
  anchor_id: u32,
  snapshots: &[(u32, CapturedImage)],
) -> Result<bool, String> {
  native::install(window, anchor_id, snapshots)?;
  Ok(true)
}

pub(crate) fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
  native::show_without_activation(window)
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
