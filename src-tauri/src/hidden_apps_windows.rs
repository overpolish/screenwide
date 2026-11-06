// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hidden apps on Windows: the apps chosen in Settings are hidden as a
//! recording starts, before the capture opens, and given back once it has
//! stopped (`screenwide_window_hiding`). What a Screenwide that stopped mid-
//! recording left hidden is given back as it starts again.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};

use screenwide_window_hiding::{HiddenProcess, Journal};
use tauri::{AppHandle, Manager};

const LIBRARY: &str = "screenwide-window-hider.dll";
const JOURNAL: &str = "hidden-windows.txt";

/// The processes hidden for the recording under way. Held across each hide
/// and give-back, so the two never run at once.
static HIDDEN: Mutex<Vec<HiddenProcess>> = Mutex::new(Vec::new());
static JOURNAL_FILE: OnceLock<Option<Journal>> = OnceLock::new();

mod shared_library;

/// Beside the app, where the bundle puts it and `pnpm dev` copies it. A Store
/// install's folder lets only Screenwide's own processes run what is in it,
/// so a packaged Screenwide loads a copy kept outside it into other apps.
fn library() -> Option<PathBuf> {
  let bundled = std::env::current_exe().ok()?.parent()?.join(LIBRARY);
  if !crate::microsoft_store::is_packaged() {
    return bundled.is_file().then_some(bundled);
  }
  static SHARED: OnceLock<Option<PathBuf>> = OnceLock::new();
  SHARED
    .get_or_init(|| shared_library::prepare(&bundled))
    .clone()
}

fn journal(app: &AppHandle) -> Option<&'static Journal> {
  JOURNAL_FILE
    .get_or_init(|| {
      app
        .path()
        .app_data_dir()
        .ok()
        .map(|directory| Journal::at(directory.join(JOURNAL)))
    })
    .as_ref()
}

/// Hides the chosen apps' windows. Blocking: called as the recording's
/// capture is opened, so a hidden window is never in its first frames.
pub(crate) fn hide_for_recording(app: &AppHandle) {
  let images: Vec<String> = crate::settings::current(app)
    .hidden_apps
    .into_iter()
    .map(|hidden| hidden.id)
    .collect();
  let (Some(library), Some(journal)) = (library(), journal(app)) else {
    return;
  };
  if images.is_empty() {
    return;
  }
  let mut hidden = HIDDEN.lock().unwrap_or_else(PoisonError::into_inner);
  if hidden.is_empty() {
    *hidden = screenwide_window_hiding::hide(&images, &library, journal);
  }
}

/// Gives the hidden windows back, off the caller's thread.
pub(crate) fn restore_after_recording(app: &AppHandle) {
  let app = app.clone();
  std::thread::spawn(move || {
    let (Some(library), Some(journal)) = (library(), journal(&app)) else {
      return;
    };
    let mut hidden = HIDDEN.lock().unwrap_or_else(PoisonError::into_inner);
    if !hidden.is_empty() {
      screenwide_window_hiding::restore(&hidden, &library, journal);
      hidden.clear();
    }
  });
}

/// Gives back what was left hidden by a Screenwide that stopped while
/// recording.
pub(crate) fn restore_left_hidden(app: &AppHandle) {
  let app = app.clone();
  std::thread::spawn(move || {
    if let (Some(library), Some(journal)) = (library(), journal(&app)) {
      let _held = HIDDEN.lock().unwrap_or_else(PoisonError::into_inner);
      screenwide_window_hiding::restore_left_hidden(&library, journal);
    }
  });
}

/// Whether one of the processes is one Screenwide may hide: Settings offers
/// only the apps it can.
pub(crate) fn can_hide_any(process_ids: &[u32]) -> bool {
  library().is_some()
    && process_ids
      .iter()
      .any(|pid| screenwide_window_hiding::can_hide(*pid))
}
