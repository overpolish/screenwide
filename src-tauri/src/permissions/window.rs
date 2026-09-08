// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, LogicalPosition, Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

use crate::windows::{self, WindowLabel};

pub fn show(app: &AppHandle) -> tauri::Result<()> {
  let window = windows::get_or_create(app, WindowLabel::Permissions, || {
    WebviewWindowBuilder::new(
      app,
      WindowLabel::Permissions.as_str(),
      WebviewUrl::App("/permissions".into()),
    )
    .title("Screenwide Permissions")
    .inner_size(540.0, 400.0)
    .center()
    .always_on_top(false)
    .closable(true)
    // The page's header leaves room for the real traffic lights, so this
    // window keeps a native title bar, drawn over its content.
    .decorations(true)
    .title_bar_style(TitleBarStyle::Overlay)
    .hidden_title(true)
    .traffic_light_position(LogicalPosition::new(20.0, 20.0))
    .resizable(false)
    .shadow(true)
    .skip_taskbar(true)
    .transparent(true)
    .effects(WindowEffectsConfig {
      color: None,
      effects: vec![Effect::UnderWindowBackground],
      radius: Some(10.0),
      state: Some(EffectState::Active),
    })
    .build()
    .inspect(|_| windows::hide_instead_of_close(app, WindowLabel::Permissions))
  })?;

  windows::show(&window, true)
}

pub fn hide(app: &AppHandle) -> tauri::Result<()> {
  if let Some(window) = app.get_webview_window(WindowLabel::Permissions.as_str()) {
    windows::hide_without_focus_transfer(&window)?;
  }

  Ok(())
}
