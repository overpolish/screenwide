// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The app's side of the licence gate the paid tools sit behind
//! (`screenwide-licensing`): it keeps the state in `license.json` beside the
//! settings, asks the licence server about keys, and answers each tool as it
//! starts. A tool the gate turns away opens Settings on the Licences pane,
//! where its key can be added or bought.

pub(crate) mod commands;
mod server;
mod store;
mod view;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use screenwide_licensing::{LicenseState, VerifyingKey};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) use screenwide_licensing::Tool;
pub(crate) use view::PaidTool;

use crate::app_windows::WindowLabel;

const FILE: &str = "license.json";
/// Tells Settings to show the Licences pane, with the tool that asked.
const OPEN_EVENT: &str = "licences://open";
/// Tells every window the licences changed, with what they now are.
const CHANGED_EVENT: &str = "licences://changed";

/// Managed on the builder and read from disk on first use, by whoever asks
/// first. Tauri builds the windows in `tauri.conf.json` before `setup`, and
/// building each WebView runs the message loop, so a page can ask for its
/// licences before `initialize` has run.
#[derive(Default)]
pub(crate) struct LicensingState(OnceLock<Licensing>);

struct Licensing {
  state: RwLock<LicenseState>,
  /// None when the app has no config folder, where nothing can be kept.
  path: Option<PathBuf>,
  server_key: VerifyingKey,
}

fn licensing(app: &AppHandle) -> &Licensing {
  app
    .state::<LicensingState>()
    .inner()
    .0
    .get_or_init(|| load(app))
}

/// Reads the state, or makes it with a new installation ID.
fn load(app: &AppHandle) -> Licensing {
  let path = app
    .path()
    .app_config_dir()
    .ok()
    .map(|folder| folder.join(FILE));
  let (state, is_new) = match path.as_deref().and_then(store::read) {
    Some(state) => (state, false),
    None => (fresh(), true),
  };
  if is_new {
    save(path.clone(), &state);
  }
  Licensing {
    state: RwLock::new(state),
    path,
    server_key: screenwide_licensing::server_key(),
  }
}

/// Loads the state, if no page has asked for it yet, and checks again with
/// the server any keys that are due.
pub(crate) fn initialize(app: &AppHandle) {
  licensing(app);
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    commands::recheck_due(&app).await;
  });
}

/// Whether `tool` may start now. The first use of any paid tool starts the
/// trial. A tool turned away opens Settings on the Licences pane.
pub(crate) fn allow(app: &AppHandle, tool: Tool) -> bool {
  if may_start(app, tool) {
    return true;
  }
  show_licences(app, Some(tool));
  false
}

/// Whether `tool` may start now, starting the trial on first use, without
/// opening Settings when it may not.
pub(crate) fn may_start(app: &AppHandle, tool: Tool) -> bool {
  let licensing = licensing(app);
  let (access, started) = {
    let Ok(mut state) = licensing.state.write() else {
      return true;
    };
    let before = state.trial_started_at;
    let access = state.begin_use(tool, now(), &licensing.server_key);
    let started = (before != state.trial_started_at).then(|| state.clone());
    (access, started)
  };
  if let Some(state) = started {
    save(licensing.path.clone(), &state);
    emit_changed(app);
  }
  access.allows_use()
}

/// Opens Settings on the Licences pane, the first time `tool` is turned away
/// in this run only: for a tool started by a gesture that may be repeated in
/// quick succession, such as Glide's.
pub(crate) fn show_licences_once(app: &AppHandle, tool: Tool) {
  static SHOWN: [AtomicBool; Tool::ALL.len()] = [const { AtomicBool::new(false) }; Tool::ALL.len()];
  let index = Tool::ALL.iter().position(|each| *each == tool).unwrap_or(0);
  if !SHOWN[index].swap(true, Ordering::Relaxed) {
    show_licences(app, Some(tool));
  }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenRequest {
  tool: Option<PaidTool>,
}

/// Brings Settings forward on the Licences pane, pointing at `tool`.
pub(crate) fn show_licences(app: &AppHandle, tool: Option<Tool>) {
  let app = app.clone();
  // Settings is shown the way its own command shows it, off the thread a
  // gesture or a shortcut arrived on.
  tauri::async_runtime::spawn(async move {
    // Built now, Settings opens on the pane; already open, its page is told.
    let open = app
      .get_webview_window(WindowLabel::Settings.as_str())
      .is_some();
    let request = OpenRequest {
      tool: tool.map(PaidTool::from),
    };
    crate::settings::show_at(&app, (!open).then_some("licences"), move |app| {
      if open {
        let _ = app.emit_to(WindowLabel::Settings.as_str(), OPEN_EVENT, request);
      }
    });
  });
}

fn emit_changed(app: &AppHandle) {
  let _ = app.emit(CHANGED_EVENT, view::current(app));
}

/// Writes the state off the calling thread, which may be one that answers a
/// gesture and must not wait on the disk.
fn save(path: Option<PathBuf>, state: &LicenseState) {
  let Some(path) = path else {
    return;
  };
  let state = state.clone();
  tauri::async_runtime::spawn_blocking(move || {
    if let Err(error) = store::write(&path, &state) {
      eprintln!("Could not save the licences: {error}");
    }
  });
}

fn fresh() -> LicenseState {
  LicenseState::new().unwrap_or_else(|_| LicenseState {
    // The system's random source failing is near impossible; the time still
    // gives an installation ID no other copy shares.
    installation_id: format!("t{:x}", now_nanos()),
    trial_started_at: None,
    activations: Vec::new(),
  })
}

fn now() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| elapsed.as_secs() as i64)
    .unwrap_or(0)
}

fn now_nanos() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| elapsed.as_nanos())
    .unwrap_or(0)
}
