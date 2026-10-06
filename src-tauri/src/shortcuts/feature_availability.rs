// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

pub(super) fn action_enabled(action: ShortcutAction) -> bool {
  match action {
    ShortcutAction::AnnotateClear | ShortcutAction::AnnotateOverlay => {
      crate::annotate::settings::enabled()
    }
    ShortcutAction::RulerOverlay => crate::ruler::settings::enabled(),
    ShortcutAction::RecognizeText => crate::text_recognition::settings::enabled(),
    // Only claimed while there is a replay to save, so the binding is free
    // for other apps the rest of the time.
    ShortcutAction::SaveReplay => replay_running(),
    _ => true,
  }
}

/// Both of live annotation's shortcuts follow the one switch.
pub(crate) fn sync_annotate_enabled(app: &AppHandle) -> Result<(), String> {
  sync_enabled(app, ShortcutAction::AnnotateOverlay)?;
  sync_enabled(app, ShortcutAction::AnnotateClear)
}

pub(crate) fn sync_ocr_enabled(app: &AppHandle) -> Result<(), String> {
  sync_enabled(app, ShortcutAction::RecognizeText)
}

/// Keep saved activation bindings while releasing them when a feature is off.
pub(crate) fn sync_ruler_enabled(app: &AppHandle) -> Result<(), String> {
  sync_enabled(app, ShortcutAction::RulerOverlay)
}

/// Whether the replay buffer is on, mirrored here because `action_enabled`
/// is asked without an app handle to read the replay state through.
static REPLAY_RUNNING: AtomicBool = AtomicBool::new(false);

fn replay_running() -> bool {
  REPLAY_RUNNING.load(Ordering::Acquire)
}

/// Claims or releases the Save Replay binding as the buffer turns on or off.
pub(crate) fn sync_replay_enabled(app: &AppHandle) -> Result<(), String> {
  REPLAY_RUNNING.store(crate::recording::replay::is_on(app), Ordering::Release);
  sync_enabled(app, ShortcutAction::SaveReplay)
}

fn sync_enabled(app: &AppHandle, action: ShortcutAction) -> Result<(), String> {
  let Some(shortcut) = shortcut_for(app, action) else {
    return Ok(());
  };
  if is_capturing() {
    return Ok(());
  }
  let parsed = shortcut
    .parse::<Shortcut>()
    .map_err(|error| error.to_string())?;
  if !action_enabled(action) {
    if app.global_shortcut().is_registered(parsed) {
      app
        .global_shortcut()
        .unregister(parsed)
        .map_err(|error| error.to_string())?;
    }
  } else if !app.global_shortcut().is_registered(parsed) {
    register_binding(app, action, &shortcut)?;
  }
  Ok(())
}
