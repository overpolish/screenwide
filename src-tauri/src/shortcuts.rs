// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod action_routing;
#[cfg(test)]
use action_routing::action_window;
#[cfg(test)]
use action_routing::preserved_capture_overlays;
#[cfg(test)]
use action_routing::requires_frontend_turn;
use action_routing::run_action;

use serde::{Deserialize, Serialize};
use std::{
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
  },
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app_windows::WindowLabel;
pub(crate) mod diagnostics;
mod registration;
use registration::register_binding;
pub use registration::{
  __cmd__begin_shortcut_capture, __cmd__end_shortcut_capture,
  __tauri_command_name_begin_shortcut_capture, __tauri_command_name_end_shortcut_capture,
};
pub use registration::{begin_shortcut_capture, end_shortcut_capture, initialize};

mod defaults;
use defaults::merge;
mod feature_availability;
pub(crate) use feature_availability::{
  sync_annotate_enabled, sync_ocr_enabled, sync_replay_enabled, sync_ruler_enabled,
};

const SHORTCUTS_FILE: &str = "shortcuts.json";
const SHORTCUT_ACTION_EVENT: &str = "global-shortcut://action";
const SCREENSHOT_SHORTCUT_REQUESTED_EVENT: &str = "screenshot-region://shortcut-requested";
static CAPTURING: AtomicBool = AtomicBool::new(false);

thread_local! {
  static IN_NATIVE_CALLBACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Runs the body of a native global-shortcut callback.
///
/// The plugin holds its shortcut registry locked for as long as a callback
/// runs, and that mutex is not reentrant: registering or releasing any
/// shortcut from inside one deadlocks the app. Anything a callback reaches
/// that needs the registry asks [`in_native_callback`] and waits for a later
/// turn instead.
pub(crate) fn during_native_callback<T>(work: impl FnOnce() -> T) -> T {
  struct Guard;
  impl Drop for Guard {
    fn drop(&mut self) {
      IN_NATIVE_CALLBACK.set(false);
    }
  }
  IN_NATIVE_CALLBACK.set(true);
  let _guard = Guard;
  work()
}

pub(crate) fn in_native_callback() -> bool {
  IN_NATIVE_CALLBACK.get()
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShortcutAction {
  ToggleRecordingBar,
  StartStopRecording,
  PauseResumeRecording,
  SaveReplay,
  TakeScreenshot,
  TakeScreenshotToClipboard,
  RecognizeText,
  AnnotateOverlay,
  AnnotateClear,
  RulerOverlay,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
  pub action: ShortcutAction,
  pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettings {
  pub bindings: Vec<ShortcutBinding>,
}

#[derive(Default)]
pub struct ShortcutSettingsState(Mutex<ShortcutSettings>);

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  Ok(app.path().app_config_dir()?.join(SHORTCUTS_FILE))
}

fn load(app: &AppHandle) -> ShortcutSettings {
  merge(
    settings_path(app)
      .ok()
      .and_then(|path| std::fs::read(path).ok())
      .and_then(|contents| serde_json::from_slice::<ShortcutSettings>(&contents).ok()),
  )
}

fn store(app: &AppHandle, settings: &ShortcutSettings) -> Result<(), String> {
  let path = settings_path(app).map_err(|error| error.to_string())?;
  if let Some(directory) = path.parent() {
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
  }
  let contents = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
  std::fs::write(path, contents).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resume_shortcut_action(app: AppHandle, action: ShortcutAction) {
  diagnostics::record("shortcut_resumed", serde_json::json!({"action": action}));
  run_action(&app, action);
}

pub fn shortcut_for(app: &AppHandle, action: ShortcutAction) -> Option<String> {
  app
    .state::<ShortcutSettingsState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .bindings
    .iter()
    .find(|binding| binding.action == action)
    .and_then(|binding| binding.shortcut.clone())
}

pub(crate) fn is_capturing() -> bool {
  CAPTURING.load(Ordering::Acquire)
}

/// Whether an action is bound to the shortcut with `id`.
pub(crate) fn assigned_to_action(app: &AppHandle, id: u32) -> bool {
  app
    .state::<ShortcutSettingsState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .bindings
    .iter()
    .filter_map(|binding| binding.shortcut.as_deref())
    .filter_map(|value| value.parse::<Shortcut>().ok())
    .any(|shortcut| shortcut.id() == id)
}

#[tauri::command]
pub fn get_shortcut_settings(state: tauri::State<'_, ShortcutSettingsState>) -> ShortcutSettings {
  state
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone()
}

#[tauri::command]
pub fn set_shortcut_binding(
  app: AppHandle,
  state: tauri::State<'_, ShortcutSettingsState>,
  action: ShortcutAction,
  shortcut: Option<String>,
) -> Result<ShortcutSettings, String> {
  let shortcut = shortcut.filter(|value| !value.trim().is_empty());
  diagnostics::record(
    "binding_requested",
    serde_json::json!({"action": action, "shortcut": shortcut}),
  );
  let mut settings = state
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  let existing = settings
    .bindings
    .iter()
    .find(|binding| binding.action == action)
    .and_then(|binding| binding.shortcut.clone());

  let requested_id = shortcut
    .as_deref()
    .map(|value| value.parse::<Shortcut>().map(|value| value.id()))
    .transpose()
    .map_err(|error| error.to_string())?;

  if settings
    .bindings
    .iter()
    .filter(|binding| binding.action != action)
    .filter_map(|binding| binding.shortcut.as_deref())
    .filter_map(|value| value.parse::<Shortcut>().ok())
    .any(|value| Some(value.id()) == requested_id)
  {
    return Err("That shortcut is already assigned to another action".to_owned());
  }
  if requested_id.is_some_and(crate::moments::settings::uses_shortcut) {
    return Err("That shortcut already places a moment".to_owned());
  }

  if let Some(existing) = existing.as_deref() {
    let result = app.global_shortcut().unregister(existing);
    diagnostics::record(
      "binding_unregister",
      serde_json::json!({"action": action, "shortcut": existing, "error": result.err().map(|error| error.to_string())}),
    );
  }
  if let Some(shortcut) = shortcut.as_deref() {
    if let Err(error) = register_binding(&app, action, shortcut) {
      if let Some(existing) = existing.as_deref() {
        let _ = register_binding(&app, action, existing);
      }
      return Err(error);
    }
  }

  if let Some(binding) = settings
    .bindings
    .iter_mut()
    .find(|binding| binding.action == action)
  {
    binding.shortcut = shortcut;
  }
  store(&app, &settings)?;
  *state
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings.clone();
  diagnostics::snapshot(&app, "binding_saved");
  crate::tray::refresh(&app);
  Ok(settings)
}

#[cfg(test)]
mod tests;
