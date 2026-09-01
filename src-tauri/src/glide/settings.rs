// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the user gets to decide about Glide. The file and the commands are
//! cross-platform on purpose: each platform reads the same settings out of the
//! same place, and only the way a modifier turns into a native flag differs.

use std::{path::PathBuf, sync::RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

const SETTINGS_FILE: &str = "glide-settings.json";
const SETTINGS_CHANGED_EVENT: &str = "glide-settings://changed";

/// The widest gap worth offering: past this the regions stop reading as a grid
/// and start reading as floating windows.
const MAXIMUM_WINDOW_GAP: u32 = 32;

/// A bare modifier key, as the two gestures are held down with. Named rather
/// than carried as a native flag so the same settings file works everywhere.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GlideModifier {
  Command,
  Option,
  Control,
  Shift,
}

/// How long the detector lets the fingers rest before it commits a transition.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GlidePacing {
  Snappy,
  Normal,
  Relaxed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GlideSettings {
  pub enabled: bool,
  pub mouse_modifier: GlideModifier,
  pub thirds_modifier: GlideModifier,
  /// The uniform gap between placed windows, in logical pixels. Outer edges are
  /// inset by the whole gap and shared edges by half each, so two adjacent
  /// windows sit exactly one gap apart.
  pub window_gap: u32,
  pub cursor_follows: bool,
  pub haptics: bool,
  pub pacing: GlidePacing,
  pub double_tap_center: bool,
}

impl Default for GlideSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      mouse_modifier: if cfg!(target_os = "windows") {
        GlideModifier::Control
      } else {
        GlideModifier::Command
      },
      thirds_modifier: GlideModifier::Shift,
      window_gap: 0,
      cursor_follows: true,
      haptics: true,
      pacing: GlidePacing::Normal,
      double_tap_center: true,
    }
  }
}

impl GlidePacing {
  pub(crate) const fn rest_ms(self) -> f64 {
    match self {
      Self::Snappy => 40.0,
      Self::Normal => 60.0,
      Self::Relaxed => 100.0,
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
    return Err("The glide and thirds modifiers must be different keys".to_owned());
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
