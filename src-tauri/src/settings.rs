// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod background_preset;
pub(crate) mod capture_exclusions;
pub(crate) mod preferences;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod shortcut_defaults;
pub(crate) mod system_cursor;
pub(crate) mod wallpapers;

pub use preferences::{current, GeneralSettingsState};

pub fn initialize(app: &tauri::AppHandle) {
  preferences::initialize(app);
  crate::annotate::settings::initialize(app);
  crate::ruler::settings::initialize(app);
  crate::moments::settings::initialize(app);
  crate::text_recognition::settings::initialize(app);
  crate::transcription::language::initialize(app);
}

use tauri::AppHandle;

use crate::app_windows::{self, on_demand, WindowLabel};

pub fn show(app: &AppHandle) {
  show_at(app, None, |_| {});
}

/// Shows Settings, then runs `then`. Settings built for this opens on the
/// pane `query` names.
pub(crate) fn show_at(
  app: &AppHandle,
  query: Option<&'static str>,
  then: impl FnOnce(&AppHandle) + Send + 'static,
) {
  crate::capture_overlays::dismiss_all(app);
  let presenter = app.clone();
  on_demand::open_at(app, WindowLabel::Settings, query, move |window| {
    let shown = window.and_then(|window| {
      app_windows::present(&presenter, &window).map_err(|error| error.to_string())
    });
    match shown {
      Ok(()) => then(&presenter),
      Err(error) => eprintln!("Could not open Settings: {error}"),
    }
  });
}

#[tauri::command]
pub fn hide_settings(app: AppHandle) -> tauri::Result<()> {
  let _ = crate::shortcuts::end_shortcut_capture(app.clone());
  on_demand::close(&app, WindowLabel::Settings);
  Ok(())
}

#[tauri::command]
pub fn show_settings(app: AppHandle) {
  show(&app);
}
