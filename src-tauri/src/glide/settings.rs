// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the user gets to decide about Glide. The file and the commands are
//! cross-platform on purpose: each platform reads the same settings out of the
//! same place, and only the way a modifier turns into a native flag differs.

use std::{path::PathBuf, sync::RwLock};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tauri::{AppHandle, Emitter, Manager};

const SETTINGS_FILE: &str = "glide-settings.json";
const SETTINGS_CHANGED_EVENT: &str = "glide-settings://changed";

/// The widest gap worth offering: past this the regions stop reading as a grid
/// and start reading as floating windows.
const MAXIMUM_WINDOW_GAP: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlideControl {
  Key(keyboard_types::Code),
  MouseMiddle,
  MouseBack,
  MouseForward,
}

impl GlideControl {
  pub const COMMAND: Self = Self::Key(keyboard_types::Code::MetaLeft);
  pub const CONTROL: Self = Self::Key(keyboard_types::Code::ControlLeft);
  pub const SHIFT: Self = Self::Key(keyboard_types::Code::ShiftLeft);
}

impl Serialize for GlideControl {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let value = match self {
      Self::Key(code) => code.to_string(),
      Self::MouseMiddle => "MouseMiddle".to_owned(),
      Self::MouseBack => "MouseBack".to_owned(),
      Self::MouseForward => "MouseForward".to_owned(),
    };
    serializer.serialize_str(&value)
  }
}

impl<'de> Deserialize<'de> for GlideControl {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = String::deserialize(deserializer)?;
    let migrated = match value.as_str() {
      "command" => "MetaLeft",
      "option" => "AltLeft",
      "control" => "ControlLeft",
      "shift" => "ShiftLeft",
      _ => value.as_str(),
    };
    match migrated {
      "MouseMiddle" => Ok(Self::MouseMiddle),
      "MouseBack" => Ok(Self::MouseBack),
      "MouseForward" => Ok(Self::MouseForward),
      _ => migrated
        .parse()
        .map(Self::Key)
        .map_err(serde::de::Error::custom),
    }
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GlideSettings {
  pub enabled: bool,
  pub mouse_modifier: GlideControl,
  pub thirds_modifier: GlideControl,
  /// The uniform gap between placed windows, in logical pixels. Outer edges are
  /// inset by the whole gap and shared edges by half each, so two adjacent
  /// windows sit exactly one gap apart.
  pub window_gap: u32,
  pub cursor_follows: bool,
  pub haptics: bool,
  pub double_tap_center: bool,
}

impl Default for GlideSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      mouse_modifier: if cfg!(target_os = "windows") {
        GlideControl::CONTROL
      } else {
        GlideControl::COMMAND
      },
      thirds_modifier: GlideControl::SHIFT,
      window_gap: 0,
      cursor_follows: true,
      haptics: true,
      double_tap_center: true,
    }
  }
}

#[derive(Default)]
pub struct GlideSettingsState(RwLock<GlideSettings>);

fn path(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|directory| directory.join(SETTINGS_FILE))
    .map_err(|error| error.to_string())
}

fn read(app: &AppHandle) -> GlideSettings {
  path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|contents| serde_json::from_slice(&contents).ok())
    .unwrap_or_default()
}

fn write(app: &AppHandle, settings: &GlideSettings) -> Result<(), String> {
  let path = path(app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let contents = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
  std::fs::write(path, contents).map_err(|error| error.to_string())
}

/// Clamps what has a range and rejects what has no sensible correction. One
/// modifier cannot drive both gestures: the grid would switch to thirds the
/// moment a glide began, and there would be no way back to halves.
fn validate(settings: &mut GlideSettings) -> Result<(), String> {
  settings.window_gap = settings.window_gap.min(MAXIMUM_WINDOW_GAP);
  if settings.mouse_modifier == settings.thirds_modifier {
    return Err("The glide and thirds controls must be different".to_owned());
  }
  if !super::platform::supports_control(settings.mouse_modifier)
    || !super::platform::supports_control(settings.thirds_modifier)
  {
    return Err("That control is not available for Glide on this platform".to_owned());
  }
  Ok(())
}

pub fn initialize(app: &AppHandle) {
  let mut settings = read(app);
  // A hand-edited file that cannot be corrected starts over rather than
  // leaving a gesture the user could not perform.
  if validate(&mut settings).is_err() {
    settings = GlideSettings::default();
  }
  *app
    .state::<GlideSettingsState>()
    .0
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
  super::platform::apply_settings(&settings);
}

pub fn current(app: &AppHandle) -> GlideSettings {
  *app
    .state::<GlideSettingsState>()
    .0
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub fn get_glide_settings(app: AppHandle) -> GlideSettings {
  current(&app)
}

#[tauri::command]
pub fn set_glide_settings(
  app: AppHandle,
  mut settings: GlideSettings,
) -> Result<GlideSettings, String> {
  validate(&mut settings)?;
  write(&app, &settings)?;
  *app
    .state::<GlideSettingsState>()
    .0
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
  // The event tap reads its own snapshot rather than this state, so the native
  // side is refreshed here, before anyone is told the settings changed.
  super::platform::apply_settings(&settings);
  let _ = app.emit(SETTINGS_CHANGED_EVENT, settings);
  Ok(settings)
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
