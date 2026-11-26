// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone as recordings and notes share it: the one a recording
//! opens, and whether the microphone of a recording or of notes is muted,
//! which every window is told of, as each island shows it.

use screenwide_capture::{microphone_choice, microphone_mute, StartRecordingOptions};
use tauri::{AppHandle, Emitter};

/// Carries whether the microphone is muted.
const MUTED_EVENT: &str = "microphone://muted";

/// A recording's options with the microphone it opens: started by its
/// shortcut, the system's default in place of a chosen microphone that is
/// not connected; started from the island, the one chosen, whose absence
/// fails the start and the island says so. The recording begins unmuted.
pub(crate) fn for_recording(
  app: &AppHandle,
  mut options: StartRecordingOptions,
) -> StartRecordingOptions {
  session_started(app);
  if options.microphone_falls_back {
    if let Some(chosen) = options.microphone_id.as_deref() {
      if let Ok(id) = microphone_choice::available_microphone(chosen) {
        options.microphone_id = Some(id);
      }
    }
  }
  options
}

/// A recording or notes begin, unmuted: a mute left on never silences the
/// next one.
pub(crate) fn session_started(app: &AppHandle) {
  set_muted(app, false);
}

/// What the mute shortcut does.
pub(crate) fn toggle_muted(app: &AppHandle) {
  set_muted(app, !microphone_mute::is_muted());
}

#[tauri::command]
pub fn microphone_muted() -> bool {
  microphone_mute::is_muted()
}

#[tauri::command]
pub fn set_microphone_muted(app: AppHandle, muted: bool) {
  set_muted(&app, muted);
}

fn set_muted(app: &AppHandle, muted: bool) {
  microphone_mute::set(muted);
  let _ = app.emit(MUTED_EVENT, muted);
}
