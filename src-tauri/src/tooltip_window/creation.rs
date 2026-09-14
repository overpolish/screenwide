// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[cfg(target_os = "windows")]
use tauri::Manager;

use crate::windows::{platform, WindowLabel};

use super::{INITIAL_HEIGHT, MAX_WIDTH};

pub(super) fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let effect = if cfg!(target_os = "windows") {
    Effect::Mica
  } else {
    Effect::UnderWindowBackground
  };
  let window = WebviewWindowBuilder::new(
    app,
    WindowLabel::Tooltip.as_str(),
    WebviewUrl::App("/tooltip".into()),
  )
  .title("Screenwide")
  .inner_size(MAX_WIDTH, INITIAL_HEIGHT)
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .minimizable(false)
  .maximizable(false)
  .resizable(false)
  .shadow(true)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .effects(WindowEffectsConfig {
    color: None,
    effects: vec![effect],
    radius: Some(8.0),
    state: Some(EffectState::Active),
  })
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
