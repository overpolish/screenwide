// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;

use tauri::ipc::CapabilityBuilder;
use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::app_windows::options::panel_material;
use crate::app_windows::{webview_window, WindowLabel};

const PREVIEW_LABEL: &str = "storybook-native";

pub fn show(app: &AppHandle, preview_url: &str) -> Result<(), Box<dyn Error>> {
  let preview_url = url::Url::parse(preview_url)?;
  // The preview is not one of the app's windows in `capabilities/`, and a
  // story listens for events as its window would: a menu's pick comes back
  // as one.
  app.add_capability(
    CapabilityBuilder::new("storybook-native")
      .window(PREVIEW_LABEL)
      .window(WindowLabel::StandaloneListbox.as_str())
      .remote(format!("{}/*", preview_url.origin().ascii_serialization()))
      .permission("core:default"),
  )?;
  create_menu_window(app, &preview_url)?;
  // A press outside a menu puts it away, as in the app.
  crate::app_windows::manage_transient_popover_dismissal(app);

  let window = WebviewWindowBuilder::new(app, PREVIEW_LABEL, WebviewUrl::External(preview_url))
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
      effects: vec![preview_effect()],
      radius: Some(10.0),
      state: Some(EffectState::Active),
    })
    .build()?;

  window.show()?;
  window.set_focus()?;
  Ok(())
}

/// The window menus and pop-up lists open in, which the app predefines in
/// `tauri.conf.json` and the preview's configuration leaves out with the rest.
/// Storybook does not serve the app's routes, so it loads the story's own page
/// told to be that window instead (`screenwide-panel` in `preview.ts`). Its
/// properties are the predefined window's.
fn create_menu_window(app: &AppHandle, preview_url: &url::Url) -> Result<(), Box<dyn Error>> {
  let mut menu_url = preview_url.clone();
  menu_url
    .query_pairs_mut()
    .append_pair("screenwide-panel", "1");
  let label = WindowLabel::StandaloneListbox;
  webview_window(app, label.as_str(), WebviewUrl::External(menu_url))
    .title(label.title())
    .inner_size(208.0, 40.0)
    .accept_first_mouse(true)
    .always_on_top(false)
    .decorations(false)
    .focused(false)
    .resizable(false)
    .shadow(true)
    .skip_taskbar(true)
    .transparent(true)
    .visible(false)
    .effects(panel_material())
    .build()?;
  Ok(())
}

fn preview_effect() -> Effect {
  if cfg!(target_os = "windows") {
    Effect::Mica
  } else {
    Effect::UnderWindowBackground
  }
}
