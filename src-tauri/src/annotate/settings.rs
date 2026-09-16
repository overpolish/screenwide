// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether live annotation is offered at all, what becomes of the annotations when
//! the overlay closes, and the dress a fresh stroke is drawn in.

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// The stroke presets the editor's arrow panel offers, which is what keeps a
/// live annotation and an editor annotation the same weight. The twin of
/// `ANNOTATION_WIDTHS` in `src/features/editor/annotations.ts`.
const MIN_WIDTH: f64 = 8.0;
const MAX_WIDTH: f64 = 48.0;
/// The palette's yellow, as in `src/features/editor/annotation-palette.ts`.
const DEFAULT_COLOR: &str = "#ffcc00";

/// What a fresh stroke is: the tag of the editor's `AnnotationShape`, so a
/// second shape in the editor becomes a second variant here.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotateShape {
  #[default]
  Arrow,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AnnotateSettings {
  pub enabled: bool,
  /// Whether annotations survive the overlay closing. Off by default: the overlay
  /// is normally toggled off to be rid of what is on screen.
  pub keep_annotations_between_sessions: bool,
  pub default_shape: AnnotateShape,
  /// `#rrggbb` or `#rrggbbaa`, as a document stores it.
  pub default_color: String,
  pub default_width: f64,
}

impl Default for AnnotateSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      keep_annotations_between_sessions: false,
      default_shape: AnnotateShape::Arrow,
      default_color: DEFAULT_COLOR.to_owned(),
      default_width: MIN_WIDTH,
    }
  }
}

/// Settings the overlay can actually draw with. A colour the compositor cannot
/// read would cost the annotation rather than the setting, so it is refused here.
fn validated(mut settings: AnnotateSettings) -> Result<AnnotateSettings, String> {
  let color = settings.default_color.to_ascii_lowercase();
  let digits = color
    .strip_prefix('#')
    .filter(|digits| matches!(digits.len(), 6 | 8))
    .filter(|digits| digits.chars().all(|digit| digit.is_ascii_hexdigit()))
    .ok_or_else(|| "That is not an annotation colour".to_owned())?;
  if !settings.default_width.is_finite()
    || !(MIN_WIDTH..=MAX_WIDTH).contains(&settings.default_width)
  {
    return Err("That is not an annotation stroke width".to_owned());
  }
  settings.default_color = format!("#{digits}");
  Ok(settings)
}

static SETTINGS: LazyLock<RwLock<AnnotateSettings>> =
  LazyLock::new(|| RwLock::new(AnnotateSettings::default()));

pub fn current() -> AnnotateSettings {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .clone()
}

pub fn enabled() -> bool {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .enabled
}

pub(super) fn keep_annotations_between_sessions() -> bool {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .keep_annotations_between_sessions
}

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join("annotate-settings.json"))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let settings = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .and_then(|settings| validated(settings).ok())
    .unwrap_or_default();
  *SETTINGS.write().unwrap_or_else(|error| error.into_inner()) = settings;
}

#[tauri::command]
pub fn get_annotate_settings() -> AnnotateSettings {
  current()
}

#[tauri::command]
pub fn set_annotate_settings(
  app: AppHandle,
  settings: AnnotateSettings,
) -> Result<AnnotateSettings, String> {
  let settings = validated(settings)?;
  let path = path(&app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
  let previous = std::mem::replace(
    &mut *SETTINGS.write().unwrap_or_else(|error| error.into_inner()),
    settings.clone(),
  );
  // The shortcut is registered against the feature being on, so it has to
  // follow the same swap the overlay does - and be put back with it.
  let result = crate::shortcuts::sync_annotate_enabled(&app)
    .and_then(|()| std::fs::write(path, bytes).map_err(|error| error.to_string()));
  if let Err(error) = result {
    *SETTINGS.write().unwrap_or_else(|error| error.into_inner()) = previous;
    let _ = crate::shortcuts::sync_annotate_enabled(&app);
    return Err(error);
  }
  if !settings.enabled {
    super::disable(&app);
  }
  crate::tray::refresh(&app);
  let _ = app.emit("annotate-settings://changed", &settings);
  Ok(settings)
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
