// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn action_enabled(action: ShortcutAction) -> bool {
  match action {
    ShortcutAction::RulerOverlay => crate::ruler::settings::enabled(),
    ShortcutAction::RecognizeText => crate::text_recognition::settings::enabled(),
    _ => true,
  }
}

pub(crate) fn sync_ocr_enabled(app: &AppHandle) -> Result<(), String> {
  sync_enabled(app, ShortcutAction::RecognizeText)
}

/// Keep saved activation bindings while releasing them when a feature is off.
pub(crate) fn sync_ruler_enabled(app: &AppHandle) -> Result<(), String> {
  sync_enabled(app, ShortcutAction::RulerOverlay)
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
