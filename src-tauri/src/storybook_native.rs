// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

pub fn show(app: &AppHandle, preview_url: &str) -> Result<(), Box<dyn Error>> {
  let effect = if cfg!(target_os = "windows") {
    Effect::Mica
  } else {
    Effect::UnderWindowBackground
  };
  let window = WebviewWindowBuilder::new(
    app,
    "storybook-native",
    WebviewUrl::External(url::Url::parse(preview_url)?),
  )
  .title("Screenwide Component Preview")
  .inner_size(520.0, 320.0)
  .min_inner_size(320.0, 200.0)
  .center()
  .always_on_top(false)
  .closable(true)
  .decorations(true)
  .resizable(true)
  .shadow(true)
  .skip_taskbar(false)
  .transparent(true)
  .effects(WindowEffectsConfig {
    color: None,
    effects: vec![effect],
    radius: Some(10.0),
    state: Some(EffectState::Active),
  })
  .build()?;

  window.show()?;
  window.set_focus()?;
  Ok(())
}
