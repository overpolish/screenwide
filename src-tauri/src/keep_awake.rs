// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake, switched from the tray or its shortcut. The choice is kept and
//! taken up again at launch, so it need not be turned on every time.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use screenwide_keep_awake::KeepAwake;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::user_work::UserActivity;

const SETTINGS_FILE: &str = "keep-awake.json";

#[derive(Default, Deserialize, Serialize)]
struct KeepAwakeSettings {
  on: bool,
}

struct Running {
  _keeper: KeepAwake,
  /// App Nap would stretch the keeper's wait past the minute a display may
  /// stay on; an otherwise idle app spends nothing by staying out of it.
  _activity: UserActivity,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

fn running() -> MutexGuard<'static, Option<Running>> {
  RUNNING.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn is_on() -> bool {
  running().is_some()
}

pub(crate) fn toggle(app: &AppHandle) {
  set(app, !is_on());
}

pub(crate) fn set(app: &AppHandle, on: bool) {
  if on && !is_on() && !screenwide_keep_awake::permitted() {
    ask_permission(app);
    return;
  }
  if on {
    start();
  } else {
    running().take();
  }
  remember(app, is_on());
  crate::tray::refresh(app);
}

/// Turns Keep Awake back on if it was on when the app last quit. Nothing is
/// asked for here: without Accessibility the permissions window opens at
/// launch anyway, and the choice stays kept for the launch that has it.
pub(crate) fn restore(app: &AppHandle) {
  let stored = settings_path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice::<KeepAwakeSettings>(&bytes).ok())
    .unwrap_or_default();
  if stored.on && screenwide_keep_awake::permitted() {
    start();
    crate::tray::refresh(app);
  }
}

fn start() {
  let mut running = running();
  if running.is_some() {
    return;
  }
  match KeepAwake::start() {
    Ok(keeper) => {
      *running = Some(Running {
        _keeper: keeper,
        _activity: UserActivity::begin("Keep Awake"),
      });
    }
    Err(error) => eprintln!("{error}"),
  }
}

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  Ok(app.path().app_config_dir()?.join(SETTINGS_FILE))
}

fn remember(app: &AppHandle, on: bool) {
  let written = settings_path(app)
    .map_err(|error| error.to_string())
    .and_then(|path| {
      if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
      }
      let bytes =
        serde_json::to_vec_pretty(&KeepAwakeSettings { on }).map_err(|error| error.to_string())?;
      std::fs::write(path, bytes).map_err(|error| error.to_string())
    });
  if let Err(error) = written {
    eprintln!("Could not save the Keep Awake setting: {error}");
  }
}

/// Sending input needs Accessibility on macOS; the permissions window says
/// how to grant it.
fn ask_permission(app: &AppHandle) {
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::permissions::show_permissions_window(app) {
    eprintln!("Could not show the permissions window for Keep Awake: {error}");
  }
  #[cfg(not(target_os = "macos"))]
  let _ = app;
}
