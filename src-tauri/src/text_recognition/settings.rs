// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::shortcut::{self, Binding};
use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  sync::{LazyLock, RwLock},
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum OcrAction {
  SelectAll,
  CopyText,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OcrSettings {
  pub enabled: bool,
  pub bindings: BTreeMap<OcrAction, Option<String>>,
}

impl Default for OcrSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      bindings: [
        (
          OcrAction::SelectAll,
          Some("CommandOrControl+KeyA".to_owned()),
        ),
        (
          OcrAction::CopyText,
          Some("CommandOrControl+KeyC".to_owned()),
        ),
      ]
      .into_iter()
      .collect(),
    }
  }
}

struct Runtime {
  settings: OcrSettings,
  bindings: Vec<(OcrAction, Binding)>,
}

impl Runtime {
  fn new(mut settings: OcrSettings) -> Result<Self, String> {
    for (action, value) in OcrSettings::default().bindings {
      settings.bindings.entry(action).or_insert(value);
    }
    let mut compiled = Vec::new();
    for (&action, value) in &settings.bindings {
      let Some(value) = value else {
        continue;
      };
      let binding = shortcut::parse(value)?;
      if compiled.iter().any(|(_, other): &(OcrAction, Binding)| {
        other.modifiers == binding.modifiers
          && (other.mac_key == binding.mac_key || other.windows_key == binding.windows_key)
      }) {
        return Err("Each OCR action needs a different shortcut".to_owned());
      }
      compiled.push((action, binding));
    }
    Ok(Self {
      settings,
      bindings: compiled,
    })
  }
  fn phase(&self, key: u16, modifiers: u32, mac: bool, repeat: bool) -> Option<u32> {
    if !self.settings.enabled || repeat {
      return None;
    }
    self
      .bindings
      .iter()
      .find(|(_, binding)| {
        (if mac {
          binding.mac_key
        } else {
          binding.windows_key
        }) == key
          && binding.modifiers == modifiers
      })
      .map(|(action, _)| match action {
        OcrAction::SelectAll => 6,
        OcrAction::CopyText => 7,
      })
  }
}

static RUNTIME: LazyLock<RwLock<Runtime>> =
  LazyLock::new(|| RwLock::new(Runtime::new(OcrSettings::default()).expect("valid OCR defaults")));

pub fn enabled() -> bool {
  RUNTIME
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .settings
    .enabled
}

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join("ocr-settings.json"))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let runtime = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .and_then(|settings| Runtime::new(settings).ok())
    .unwrap_or_else(|| Runtime::new(OcrSettings::default()).expect("valid OCR defaults"));
  *RUNTIME.write().unwrap_or_else(|error| error.into_inner()) = runtime;
}

#[tauri::command]
pub fn get_ocr_settings() -> OcrSettings {
  RUNTIME
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .settings
    .clone()
}

#[tauri::command]
pub fn set_ocr_settings(app: AppHandle, settings: OcrSettings) -> Result<OcrSettings, String> {
  let next = Runtime::new(settings)?;
  let settings = next.settings.clone();
  let path = path(&app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
  let previous = std::mem::replace(
    &mut *RUNTIME.write().unwrap_or_else(|error| error.into_inner()),
    next,
  );
  let result = crate::shortcuts::sync_ocr_enabled(&app)
    .and_then(|()| std::fs::write(path, bytes).map_err(|error| error.to_string()));
  if let Err(error) = result {
    *RUNTIME.write().unwrap_or_else(|error| error.into_inner()) = previous;
    let _ = crate::shortcuts::sync_ocr_enabled(&app);
    return Err(error);
  }
  if !settings.enabled {
    super::dismiss(&app);
  }
  crate::tray::refresh(&app);
  let _ = app.emit("ocr-settings://changed", &settings);
  Ok(settings)
}

pub(crate) fn key_phase(key: u16, modifiers: u32, mac: bool, repeat: bool) -> Option<u32> {
  if crate::shortcuts::is_capturing() {
    return None;
  }
  RUNTIME
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .phase(key, modifiers, mac, repeat)
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn screenwide_ocr_key_phase(key: u16, modifiers: u32, repeat: bool) -> u32 {
  key_phase(key, modifiers, true, repeat).unwrap_or(0)
}

#[cfg(test)]
mod tests;
