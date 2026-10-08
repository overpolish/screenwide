// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the editor needs in place before any of its windows opens.

use super::*;

pub fn initialize(app: &AppHandle) {
  let state = app.state::<EditorState>();
  *state
    .recording_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_recording_output(app);
  *state
    .screenshot_output
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_output(app);
  *state
    .screenshot_background_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_background_radius(app);
  *state
    .screenshot_radius_percent
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_screenshot_radius(app);
  *state
    .cursor_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_cursor_effects(app);
  *state
    .keyboard_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_keyboard_effects(app);
  *state
    .recording_choices
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = load_recording_choices(app);
  *state
    .screenshot_delete_project_after_export
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
    load_screenshot_delete_project_after_export(app);
  crate::project::sweep_cancelled(app);
  // Off the launch path: what has waited 30 days in Recently Deleted goes to
  // the Trash, which can take a moment for a large project.
  let expiring = app.clone();
  tauri::async_runtime::spawn_blocking(move || crate::project::expire(&expiring));
  if let Ok(data) = app.path().app_data_dir() {
    super::images::store::initialize(&data);
  }
}
