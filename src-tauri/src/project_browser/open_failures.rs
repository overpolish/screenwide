// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Projects that would not open when asked from outside the app: by Finder,
//! Explorer or a second launch. Nothing of the app's may be on screen then,
//! so the browser comes out and says why on its island, as it does for a
//! project opened from it.
//!
//! The reasons wait here until the browser takes them, since at launch its
//! page may not be listening yet.

use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, Emitter, Manager};

use crate::app_windows::WindowLabel;

/// Sent to the browser when a reason is waiting for it.
const OPEN_FAILED_EVENT: &str = "projects://open-failed";

#[derive(Default)]
pub struct OpenFailures(Mutex<Vec<String>>);

fn reasons(app: &AppHandle) -> MutexGuard<'_, Vec<String>> {
  app
    .state::<OpenFailures>()
    .inner()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Shows the browser, saying why a project could not open. Callable from any
/// thread; the window is handled on the main thread.
pub fn report(app: &AppHandle, reason: String) {
  {
    let mut reasons = reasons(app);
    if !reasons.contains(&reason) {
      reasons.push(reason);
    }
  }
  let task_app = app.clone();
  let _ = app.run_on_main_thread(move || {
    if let Err(error) = super::show(&task_app) {
      eprintln!("Could not show the project browser for a failed open: {error}");
    }
    let _ = task_app.emit_to(WindowLabel::Projects.as_str(), OPEN_FAILED_EVENT, ());
  });
}

/// The reasons waiting, oldest first, for the browser to show; each is given
/// once.
#[tauri::command]
pub fn take_project_open_failures(app: AppHandle) -> Vec<String> {
  std::mem::take(&mut *reasons(&app))
}
