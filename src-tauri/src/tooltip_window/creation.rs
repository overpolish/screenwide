// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, WebviewUrl, WebviewWindow};

#[cfg(target_os = "windows")]
use tauri::Manager;

use crate::app_windows::{platform, WindowLabel};

use super::{INITIAL_HEIGHT, MAX_WIDTH};

/// The tooltip is drawn in the island's black by its page, as a menu opened
/// from an island is, so the window carries no material under it and no
/// shadow: AppKit draws that with a light rim, which reads as a border on
/// the black.
pub(super) fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let window = crate::app_windows::webview_window(
    app,
    WindowLabel::Tooltip.as_str(),
    WebviewUrl::App("/tooltip".into()),
  )
  .inner_size(MAX_WIDTH, INITIAL_HEIGHT)
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .minimizable(false)
  .maximizable(false)
  .resizable(false)
  .shadow(false)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .build()?;

  platform::initialize_tooltip(&window)?;
  window.set_ignore_cursor_events(true)?;
  Ok(window)
}

#[cfg(target_os = "windows")]
pub(super) fn initialize(app: &AppHandle) -> tauri::Result<()> {
  if app
    .get_webview_window(WindowLabel::Tooltip.as_str())
    .is_none()
  {
    build(app)?;
  }
  Ok(())
}
