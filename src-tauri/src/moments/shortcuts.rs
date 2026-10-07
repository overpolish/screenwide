// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The kinds' shortcuts, claimed only while a recording runs so their keys
//! belong to other apps the rest of the time.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::settings;
use super::voice::{self, Voice};

const PLACED_EVENT: &str = "moments://placed";
const RELEASED_EVENT: &str = "moments://released";

/// The shortcuts claimed for kinds now, so they can be given back.
static REGISTERED: Mutex<Vec<Shortcut>> = Mutex::new(Vec::new());

/// Kinds whose shortcut is held down, and whether the hold is listening for
/// a note. A held key may repeat its press, and one hold is one moment.
static HELD: LazyLock<Mutex<HashMap<String, bool>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlacedPayload {
  color: String,
  name: String,
  voice: Voice,
}

/// Claims the kinds' shortcuts while a recording runs and gives them back
/// otherwise, with the kinds Settings has now.
///
/// Always on a later turn: recording transitions arrive from inside native
/// shortcut callbacks (Start/Stop, Pause/Resume), and the plugin's registry is
/// locked for the length of one. Each turn reads the state it acts on when it
/// runs, so turns that land out of order still settle on the latest.
pub(crate) fn sync(app: &AppHandle) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    reconcile(&app);
  });
}

fn reconcile(app: &AppHandle) {
  let mut registered = REGISTERED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  for shortcut in registered.drain(..) {
    let _ = app.global_shortcut().unregister(shortcut);
  }
  // A key held through this change will never report its release, so its
  // hold ends here, and the note it was listening for with it.
  let held = std::mem::take(&mut *HELD.lock().unwrap_or_else(|poisoned| poisoned.into_inner()));
  if held.values().any(|listening| *listening) {
    voice::end(Instant::now());
  }
  if !held.is_empty() {
    let _ = app.emit(RELEASED_EVENT, ());
  }
  let hidden = if crate::recording::is_recording(app) && !crate::shortcuts::is_capturing() {
    register_kinds(app, &mut registered)
  } else {
    Vec::new()
  };
  crate::recording::keyboard::set_hidden_shortcuts(&hidden);
}

/// Claims every kind's shortcut that is free, into `registered`, and returns
/// the shortcuts the keyboard sidecar has to leave out.
fn register_kinds(app: &AppHandle, registered: &mut Vec<Shortcut>) -> Vec<String> {
  let mut claimed = Vec::new();
  for kind in settings::current().kinds {
    let Some(value) = kind.shortcut else {
      continue;
    };
    let Ok(shortcut) = value.parse::<Shortcut>() else {
      continue;
    };
    // Settings refuses a kind the keys of an action, but a kind kept from
    // before that action was given them can still hold them; the action wins.
    if crate::shortcuts::assigned_to_action(app, shortcut.id()) {
      continue;
    }
    let kind_id = kind.id;
    let result = app
      .global_shortcut()
      .on_shortcut(shortcut, move |app, _, event| {
        on_shortcut(app, &kind_id, event.state());
      });
    match result {
      Ok(()) => {
        registered.push(shortcut);
        claimed.push(value);
      }
      Err(error) => eprintln!("Could not claim the moment shortcut {value}: {error}"),
    }
  }
  claimed
}

fn on_shortcut(app: &AppHandle, kind_id: &str, state: ShortcutState) {
  let at = Instant::now();
  let mut held = HELD.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
  if state == ShortcutState::Released {
    if let Some(listening) = held.remove(kind_id) {
      drop(held);
      if listening {
        voice::end(at);
      }
      let _ = app.emit(RELEASED_EVENT, ());
    }
    return;
  }
  if held.contains_key(kind_id) {
    return;
  }
  let Some(kind) = settings::kind(kind_id) else {
    return;
  };
  let Some(recorder) = super::recorder::current() else {
    return;
  };
  let Some(moment) = recorder.record(&kind, at) else {
    return;
  };
  let voice = voice::begin(app, recorder, moment, at);
  held.insert(kind_id.to_owned(), matches!(voice, Voice::Recording));
  drop(held);
  let _ = app.emit(
    PLACED_EVENT,
    PlacedPayload {
      color: kind.color,
      name: kind.name,
      voice,
    },
  );
}
