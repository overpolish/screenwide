// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the moment settings are kept, and the commands Settings reads and
//! changes them through.

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;

use screenwide_moments::settings::{self, MomentSettings};

const SETTINGS_FILE: &str = "moments-settings.json";

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join(SETTINGS_FILE))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  if let Ok(path) = path(app) {
    settings::load(&path);
  }
}

#[tauri::command]
pub fn get_moment_settings() -> MomentSettings {
  settings::current()
}

#[tauri::command]
pub fn set_moment_settings(
  app: AppHandle,
  settings: MomentSettings,
) -> Result<MomentSettings, String> {
  let settings = settings::validate(settings)?;
  let taken = settings
    .kinds
    .iter()
    .filter_map(|kind| kind.shortcut.as_deref())
    .filter_map(|value| value.parse::<Shortcut>().ok())
    .any(|shortcut| crate::shortcuts::assigned_to_action(&app, shortcut.id()));
  if taken {
    return Err("That shortcut is already assigned to another action".to_owned());
  }
  let transcribing = settings::save(&path(&app)?, settings.clone())?.transcribe_notes;
  // A recording under way picks up the new shortcuts at once.
  super::shortcuts::sync(&app);
  // Notes show their transcripts, or stop offering them, at once.
  if transcribing != settings.transcribe_notes {
    crate::transcription::notes::changed(&app);
  }
  Ok(settings)
}
