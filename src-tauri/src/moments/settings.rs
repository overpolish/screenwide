// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;

const SETTINGS_FILE: &str = "moments-settings.json";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentKind {
  /// Stable across renames, so a recording's moments keep their kind.
  pub id: String,
  pub name: String,
  /// `#rrggbb`.
  pub color: String,
  /// The colour last chosen outside the palette, kept so it can be chosen
  /// again after a palette colour, or nothing if none ever was.
  #[serde(default)]
  pub custom_color: Option<String>,
  #[serde(default)]
  pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentSettings {
  pub kinds: Vec<MomentKind>,
}

impl Default for MomentSettings {
  fn default() -> Self {
    let kind = |id: &str, name: &str, color: &str, digit: u8| MomentKind {
      color: color.to_owned(),
      custom_color: None,
      id: id.to_owned(),
      name: name.to_owned(),
      shortcut: Some(format!("CommandOrControl+Shift+Digit{digit}")),
    };
    Self {
      kinds: vec![
        kind("funny", "Funny", "#ffcc00", 1),
        kind("notable", "Notable", "#0088ff", 2),
      ],
    }
  }
}

static SETTINGS: LazyLock<RwLock<MomentSettings>> =
  LazyLock::new(|| RwLock::new(MomentSettings::default()));

pub(crate) fn current() -> MomentSettings {
  SETTINGS
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone()
}

/// The kind with `id`, as Settings has it now.
pub(super) fn kind(id: &str) -> Option<MomentKind> {
  SETTINGS
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .kinds
    .iter()
    .find(|kind| kind.id == id)
    .cloned()
}

/// Whether a kind is bound to the shortcut with `id`.
pub(crate) fn uses_shortcut(id: u32) -> bool {
  SETTINGS
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .kinds
    .iter()
    .filter_map(|kind| kind.shortcut.as_deref())
    .filter_map(|value| value.parse::<Shortcut>().ok())
    .any(|shortcut| shortcut.id() == id)
}

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join(SETTINGS_FILE))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let stored = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice::<MomentSettings>(&bytes).ok())
    .and_then(|settings| validate(settings).ok());
  if let Some(settings) = stored {
    *SETTINGS
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
  }
}

/// `settings` tidied, or why they cannot be kept: every kind needs an id of
/// its own, a name and a colour, and no two kinds may share a shortcut.
pub(super) fn validate(mut settings: MomentSettings) -> Result<MomentSettings, String> {
  let mut ids = HashSet::new();
  let mut shortcuts = HashSet::new();
  for kind in &mut settings.kinds {
    kind.name = kind.name.trim().to_owned();
    if kind.id.is_empty() || !ids.insert(kind.id.clone()) {
      return Err("Each moment kind needs an id of its own".to_owned());
    }
    if kind.name.is_empty() {
      return Err("Each moment kind needs a name".to_owned());
    }
    if !is_hex_color(&kind.color)
      || kind
        .custom_color
        .as_deref()
        .is_some_and(|color| !is_hex_color(color))
    {
      return Err(format!("{} has no valid colour", kind.name));
    }
    kind.shortcut = kind
      .shortcut
      .take()
      .filter(|value| !value.trim().is_empty());
    if let Some(value) = kind.shortcut.as_deref() {
      let shortcut = value
        .parse::<Shortcut>()
        .map_err(|error| error.to_string())?;
      if !shortcuts.insert(shortcut.id()) {
        return Err("Each moment kind needs a different shortcut".to_owned());
      }
    }
  }
  Ok(settings)
}

fn is_hex_color(value: &str) -> bool {
  value.len() == 7
    && value.starts_with('#')
    && value[1..].chars().all(|digit| digit.is_ascii_hexdigit())
}

#[tauri::command]
pub fn get_moment_settings() -> MomentSettings {
  current()
}

#[tauri::command]
pub fn set_moment_settings(
  app: AppHandle,
  settings: MomentSettings,
) -> Result<MomentSettings, String> {
  let settings = validate(settings)?;
  let taken = settings
    .kinds
    .iter()
    .filter_map(|kind| kind.shortcut.as_deref())
    .filter_map(|value| value.parse::<Shortcut>().ok())
    .any(|shortcut| crate::shortcuts::assigned_to_action(&app, shortcut.id()));
  if taken {
    return Err("That shortcut is already assigned to another action".to_owned());
  }
  let path = path(&app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
  std::fs::write(path, bytes).map_err(|error| error.to_string())?;
  *SETTINGS
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings.clone();
  // A recording under way picks up the new shortcuts at once.
  super::shortcuts::sync(&app);
  Ok(settings)
}
