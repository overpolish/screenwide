// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake and Keep Screen On, switched from the tray or their shortcuts.
//! Keeping the screen on keeps the computer awake too: turning Keep Screen
//! On on turns Keep Awake on, and turning Keep Awake off ends both. The
//! choices are kept and taken up again at launch, so they need not be turned
//! on every time.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use screenwide_keep_awake::{KeepAwake, ScreenOn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "keep-awake.json";
const REASON: &str = "Keep Awake";
const SCREEN_REASON: &str = "Keep Screen On";

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeepAwakeSettings {
  on: bool,
  #[serde(default)]
  screen_on: bool,
}

struct Running {
  _awake: KeepAwake,
  screen: Option<ScreenOn>,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

fn running() -> MutexGuard<'static, Option<Running>> {
  RUNNING.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn is_on() -> bool {
  running().is_some()
}

pub(crate) fn is_screen_on() -> bool {
  running()
    .as_ref()
    .is_some_and(|running| running.screen.is_some())
}

pub(crate) fn toggle(app: &AppHandle) {
  set(app, !is_on());
}

pub(crate) fn toggle_screen_on(app: &AppHandle) {
  set_screen_on(app, !is_screen_on());
}

pub(crate) fn set(app: &AppHandle, on: bool) {
  if on {
    start();
  } else {
    running().take();
  }
  changed(app);
}

pub(crate) fn set_screen_on(app: &AppHandle, on: bool) {
  if on {
    start();
    start_screen();
  } else if let Some(running) = running().as_mut() {
    running.screen = None;
  }
  changed(app);
}

/// Turns back on what was on when the app last quit.
pub(crate) fn restore(app: &AppHandle) {
  let stored = settings_path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice::<KeepAwakeSettings>(&bytes).ok())
    .unwrap_or_default();
  if stored.on {
    start();
    if stored.screen_on {
      start_screen();
    }
    crate::tray::refresh(app);
  }
}

fn start() {
  let mut running = running();
  if running.is_some() {
    return;
  }
  match KeepAwake::start(REASON) {
    Ok(awake) => {
      *running = Some(Running {
        _awake: awake,
        screen: None,
      });
    }
    Err(error) => eprintln!("{error}"),
  }
}

fn start_screen() {
  let mut running = running();
  let Some(running) = running.as_mut() else {
    return;
  };
  if running.screen.is_some() {
    return;
  }
  match ScreenOn::start(SCREEN_REASON) {
    Ok(screen) => running.screen = Some(screen),
    Err(error) => eprintln!("{error}"),
  }
}

fn changed(app: &AppHandle) {
  remember(app);
  crate::tray::refresh(app);
}

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  Ok(app.path().app_config_dir()?.join(SETTINGS_FILE))
}

fn remember(app: &AppHandle) {
  let settings = KeepAwakeSettings {
    on: is_on(),
    screen_on: is_screen_on(),
  };
  let written = settings_path(app)
    .map_err(|error| error.to_string())
    .and_then(|path| {
      if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
      }
      let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
      std::fs::write(path, bytes).map_err(|error| error.to_string())
    });
  if let Err(error) = written {
    eprintln!("Could not save the Keep Awake setting: {error}");
  }
}
