// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod bindings;

use crate::osc::shortcut::{self, Binding};
use bindings::RulerAction;
use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  sync::{LazyLock, RwLock},
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RulerSettings {
  pub enabled: bool,
  pub bindings: BTreeMap<RulerAction, Option<String>>,
}

impl Default for RulerSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      bindings: bindings::defaults(),
    }
  }
}

struct Runtime {
  settings: RulerSettings,
  bindings: Vec<(RulerAction, Binding)>,
}

impl Runtime {
  fn new(mut settings: RulerSettings) -> Result<Self, String> {
    for (action, value) in bindings::defaults() {
      settings.bindings.entry(action).or_insert(value);
    }
    let mut compiled = Vec::new();
    for (&action, value) in &settings.bindings {
      let Some(value) = value else {
        continue;
      };
      let binding = shortcut::parse(value)?;
      if compiled.iter().any(|(_, other): &(RulerAction, Binding)| {
        other.modifiers == binding.modifiers
          && (other.mac_key == binding.mac_key || other.windows_key == binding.windows_key)
      }) {
        return Err("Each Ruler action needs a different shortcut".to_owned());
      }
      compiled.push((action, binding));
    }
    Ok(Self {
      settings,
      bindings: compiled,
    })
  }
}

static RUNTIME: LazyLock<RwLock<Runtime>> = LazyLock::new(|| {
  RwLock::new(Runtime::new(RulerSettings::default()).expect("valid ruler defaults"))
});

pub fn current() -> RulerSettings {
  RUNTIME
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .settings
    .clone()
}

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
    .map(|path| path.join("ruler-settings.json"))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let runtime = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .and_then(|settings| Runtime::new(settings).ok())
    .unwrap_or_else(|| Runtime::new(RulerSettings::default()).expect("valid ruler defaults"));
  *RUNTIME.write().unwrap_or_else(|error| error.into_inner()) = runtime;
}

#[tauri::command]
pub fn get_ruler_settings() -> RulerSettings {
  current()
}

#[tauri::command]
pub fn set_ruler_settings(
  app: AppHandle,
  settings: RulerSettings,
) -> Result<RulerSettings, String> {
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
  let result = crate::shortcuts::sync_ruler_enabled(&app)
    .and_then(|()| std::fs::write(path, bytes).map_err(|error| error.to_string()));
  if let Err(error) = result {
    *RUNTIME.write().unwrap_or_else(|error| error.into_inner()) = previous;
    let _ = crate::shortcuts::sync_ruler_enabled(&app);
    return Err(error);
  }
  if !settings.enabled {
    super::dismiss(&app);
  }
  crate::tray::refresh(&app);
  let _ = app.emit("ruler-settings://changed", &settings);
  Ok(settings)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyCommand {
  pub phase: u32,
  pub release: Option<u32>,
}

pub(crate) fn key_command(
  key: u16,
  modifiers: u32,
  mac: bool,
  repeat: bool,
  latched: bool,
) -> Option<KeyCommand> {
  if crate::shortcuts::is_capturing() {
    return None;
  }
  let runtime = RUNTIME.read().unwrap_or_else(|error| error.into_inner());
  resolve(&runtime, key, modifiers, mac, repeat, latched)
}

fn resolve(
  runtime: &Runtime,
  key: u16,
  modifiers: u32,
  mac: bool,
  repeat: bool,
  latched: bool,
) -> Option<KeyCommand> {
  if !runtime.settings.enabled {
    return None;
  }
  let (action, _) = runtime
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
    .or_else(|| {
      // Option/Alt changes probe boundaries while stamping. Prefer an exact
      // shortcut above, then allow it alongside the configured stamp binding.
      runtime.bindings.iter().find(|(action, binding)| {
        matches!(
          action,
          RulerAction::StampHorizontal | RulerAction::StampVertical
        ) && (if mac {
          binding.mac_key
        } else {
          binding.windows_key
        }) == key
          && modifiers & 4 != 0
          && binding.modifiers == modifiers & !4
      })
    })
    .or_else(|| {
      // Preserve familiar alternative keys only while the action retains its
      // default binding. An explicit binding above always takes precedence.
      runtime
        .bindings
        .iter()
        .find(|(action, binding)| match action {
          RulerAction::DeleteMeasurement => {
            binding.mac_key == 0x33
              && binding.modifiers == 0
              && modifiers == 0
              && key == if mac { 0x75 } else { 0x2e }
          }
          RulerAction::Redo => {
            binding.mac_key == 0x06
              && binding.modifiers == (if cfg!(target_os = "macos") { 1 } else { 2 }) | 8
              && modifiers == binding.modifiers & !8
              && key == if mac { 0x10 } else { 0x59 }
          }
          _ => false,
        })
    })?;
  if (action.release().is_some() && (repeat || latched))
    || (repeat
      && matches!(
        action,
        RulerAction::CycleTolerance | RulerAction::ToggleCenterlines
      ))
  {
    return None;
  }
  Some(KeyCommand {
    phase: action.phase(),
    release: action.release(),
  })
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn screenwide_ruler_key_phase(
  key: u16,
  modifiers: u32,
  repeat: bool,
  latched: bool,
) -> u32 {
  key_command(key, modifiers, true, repeat, latched).map_or(0, |command| command.phase)
}

#[cfg(test)]
mod tests;
