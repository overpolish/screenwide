// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! While one of Screenwide's menus is open, the window server holds back the
//! hot keys Screenwide registered until the menu closes, so a shortcut
//! pressed to capture the menu would only run once it had gone. While a menu
//! is open a key tap takes Screenwide's shortcuts instead and runs them at
//! once; a capture tool's then freezes the menu into its picture
//! (`menu_freeze`). Taken at the tap, the key never reaches the window
//! server, so the held-back hot key does not fire again as the menu closes.

use std::sync::{Mutex, PoisonError};

use screenwide_osc::shortcut::Binding;
use screenwide_system_keys::KeyTap;

use super::*;

static TAP: Mutex<Option<KeyTap>> = Mutex::new(None);

/// A menu of ours opened or closed. On the main thread.
pub(crate) fn menu_changed(app: &AppHandle, open: bool) {
  let mut tap = TAP.lock().unwrap_or_else(PoisonError::into_inner);
  if !open {
    *tap = None;
    return;
  }
  if tap.is_some() {
    return;
  }
  let bindings = bindings(app);
  if bindings.is_empty() {
    return;
  }
  let app = app.clone();
  let started = KeyTap::start(move |key, held| {
    let Some(action) = bindings
      .iter()
      .find(|(binding, _)| binding.mac_key == key && binding.modifiers == held)
      .map(|(_, action)| *action)
    else {
      return false;
    };
    diagnostics::record(
      "menu_shortcut_pressed",
      serde_json::json!({"action": action}),
    );
    let main = app.clone();
    let _ = app.run_on_main_thread(move || run_action(&main, action));
    true
  });
  match started {
    Ok(started) => *tap = Some(started),
    Err(error) => eprintln!("Could not take shortcuts while a menu is open: {error}"),
  }
}

/// Every shortcut set, in the form the tap compares. Dictation is left to
/// its hot key: it acts on the key's release as well as its press.
fn bindings(app: &AppHandle) -> Vec<(Binding, ShortcutAction)> {
  app
    .state::<ShortcutSettingsState>()
    .0
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .bindings
    .iter()
    .filter(|binding| binding.action != ShortcutAction::Dictate)
    .filter_map(|binding| {
      let parsed = screenwide_osc::shortcut::parse(binding.shortcut.as_deref()?).ok()?;
      Some((parsed, binding.action))
    })
    .collect()
}
