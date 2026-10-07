// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "transcription.json";

/// The computer's own language, whatever it is at the time.
const SYSTEM: &str = "system";
/// Detected from the speech itself.
const AUTO: &str = "auto";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptionSettings {
  /// [`SYSTEM`], [`AUTO`], or a language code such as `en`.
  language: String,
}

static LANGUAGE: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(SYSTEM.to_owned()));

pub(crate) fn current() -> String {
  LANGUAGE
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone()
}

/// The computer's language as a code: `en` from `en-GB`. English when it
/// cannot be told.
pub(crate) fn system() -> String {
  sys_locale::get_locale()
    .as_deref()
    .and_then(|locale| locale.split(['-', '_']).next())
    .filter(|code| is_code(code))
    .unwrap_or("en")
    .to_lowercase()
}

/// What to ask the transcriber for: a code, or nothing to detect it.
pub(super) fn resolved(setting: &str) -> Option<String> {
  match setting {
    AUTO => None,
    SYSTEM => Some(system()),
    code => Some(code.to_owned()),
  }
}

/// A two- or three-letter code, as Whisper names its languages.
pub(super) fn is_code(value: &str) -> bool {
  (2..=3).contains(&value.len()) && value.chars().all(|letter| letter.is_ascii_alphabetic())
}

pub(super) fn is_valid(value: &str) -> bool {
  value == SYSTEM || value == AUTO || (is_code(value) && value == value.to_lowercase())
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
    .and_then(|bytes| serde_json::from_slice::<TranscriptionSettings>(&bytes).ok())
    .filter(|settings| is_valid(&settings.language));
  if let Some(settings) = stored {
    *LANGUAGE
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings.language;
  }
}

pub(super) fn set(app: &AppHandle, language: &str) -> Result<(), String> {
  if !is_valid(language) {
    return Err(format!("{language} is not a language code"));
  }
  let path = path(app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let settings = TranscriptionSettings {
    language: language.to_owned(),
  };
  let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
  std::fs::write(path, bytes).map_err(|error| error.to_string())?;
  *LANGUAGE
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings.language;
  Ok(())
}
