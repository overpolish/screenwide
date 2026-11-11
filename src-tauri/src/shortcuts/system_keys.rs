// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's own screenshot keys, running Screenwide's tools in place of
//! the system's while "Use the system's screenshot keys" is on.

use screenwide_system_keys::{SystemKey, SystemKeys};

use super::*;

/// Tells the island, once, which of the system's keys it just answered.
const ANNOUNCE_EVENT: &str = "system-keys://announce";

/// Each key, written as the settings window records keys, and what it runs.
const TAKEN: &[(SystemKey, &str, ShortcutAction)] = &[
  (
    SystemKey::ScreenClip,
    "Super+Shift+KeyS",
    ShortcutAction::TakeScreenshot,
  ),
  (
    SystemKey::PrintScreen,
    "PrintScreen",
    ShortcutAction::TakeScreenshot,
  ),
  (
    SystemKey::ScreenRecord,
    "Super+Shift+KeyR",
    ShortcutAction::ToggleRecordingControls,
  ),
  (
    SystemKey::TextExtractor,
    "Super+Shift+KeyT",
    ShortcutAction::RecognizeText,
  ),
];

static HOOK: Mutex<Option<SystemKeys>> = Mutex::new(None);

/// One of the system's keys and the action it runs while taken.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SystemShortcut {
  pub action: ShortcutAction,
  pub shortcut: String,
}

/// The system's keys Screenwide can take on this platform; none where it
/// cannot, so Settings offers nothing there.
pub(crate) fn system_shortcuts() -> Vec<SystemShortcut> {
  if !screenwide_system_keys::supported() {
    return Vec::new();
  }
  TAKEN
    .iter()
    .map(|&(_, shortcut, action)| SystemShortcut {
      action,
      shortcut: shortcut.to_owned(),
    })
    .collect()
}

#[tauri::command]
pub fn set_replace_system_keys(
  app: AppHandle,
  state: tauri::State<'_, ShortcutSettingsState>,
  replace: bool,
) -> Result<ShortcutSettings, String> {
  let mut settings = state
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  settings.replace_system_keys = replace;
  store(&app, &settings)?;
  let saved = settings.clone();
  drop(settings);
  sync(&app, replace);
  Ok(saved)
}

/// Takes the keys while `replace` is on and gives them back once it is off.
pub(super) fn sync(app: &AppHandle, replace: bool) {
  let mut hook = HOOK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
  if !replace || !screenwide_system_keys::supported() {
    *hook = None;
    return;
  }
  if hook.is_some() {
    return;
  }
  let app = app.clone();
  match SystemKeys::start(move |key| take(&app, key)) {
    Ok(keys) => *hook = Some(keys),
    Err(error) => {
      diagnostics::record("system_keys_start", serde_json::json!({ "error": error }));
      eprintln!("{error}");
    }
  }
}

/// Whether the shortcut with `id` is one of the system's keys `settings`
/// has taken, which no action may also have.
pub(super) fn taken(settings: &ShortcutSettings, id: u32) -> bool {
  settings.replace_system_keys
    && screenwide_system_keys::supported()
    && TAKEN.iter().any(|&(_, shortcut, _)| {
      shortcut
        .parse::<Shortcut>()
        .is_ok_and(|shortcut| shortcut.id() == id)
    })
}

/// Asked on the keyboard hook's thread, which holds every keystroke until it
/// answers: decides, and leaves the work to the main thread. A key whose
/// tool is off, or pressed while a shortcut field is recording, stays the
/// system's.
fn take(app: &AppHandle, key: SystemKey) -> bool {
  let Some(&(_, shortcut, action)) = TAKEN.iter().find(|(taken, ..)| *taken == key) else {
    return false;
  };
  if is_capturing() || !feature_availability::action_enabled(action) {
    return false;
  }
  let main = app.clone();
  app
    .run_on_main_thread(move || {
      diagnostics::record(
        "system_key_pressed",
        serde_json::json!({ "action": action, "shortcut": shortcut }),
      );
      run_action(&main, action);
      announce_once(&main, shortcut);
    })
    .is_ok()
}

/// The first time one of the system's keys runs a tool, the island says so
/// and where to turn it off.
fn announce_once(app: &AppHandle, shortcut: &str) {
  let state = app.state::<ShortcutSettingsState>();
  let mut settings = state
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if settings.system_keys_announced {
    return;
  }
  settings.system_keys_announced = true;
  if let Err(error) = store(app, &settings) {
    eprintln!("Could not save shortcut settings: {error}");
  }
  drop(settings);
  let _ = app.emit_to(WindowLabel::Island.as_str(), ANNOUNCE_EVENT, shortcut);
}
