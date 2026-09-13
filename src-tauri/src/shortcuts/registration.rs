// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn register_binding(
  app: &AppHandle,
  action: ShortcutAction,
  shortcut: &str,
) -> Result<(), String> {
  if !feature_availability::action_enabled(action) {
    diagnostics::record(
      "registration_skipped",
      serde_json::json!({"action": action, "shortcut": shortcut, "reason": "feature_disabled"}),
    );
    return Ok(());
  }
  let result = (|| {
    let parsed = shortcut
      .parse::<Shortcut>()
      .map_err(|error| error.to_string())?;
    app
      .global_shortcut()
      .on_shortcut(parsed, move |app, _, event| {
        if event.state() == ShortcutState::Pressed {
          diagnostics::record(
            "native_shortcut_pressed",
            serde_json::json!({"action": action, "capturing": is_capturing()}),
          );
          run_action(app, action);
        }
      })
      .map_err(|error| error.to_string())
  })();
  diagnostics::record(
    "registration_result",
    serde_json::json!({"action": action, "shortcut": shortcut, "error": result.as_ref().err()}),
  );
  result
}

pub fn initialize(app: &AppHandle) {
  let settings = load(app);
  for binding in &settings.bindings {
    if let Some(shortcut) = binding.shortcut.as_deref() {
      if let Err(error) = register_binding(app, binding.action, shortcut) {
        eprintln!("Could not register {shortcut}: {error}");
      }
    }
  }
  *app
    .state::<ShortcutSettingsState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
  diagnostics::snapshot(app, "startup_registration_complete");
  crate::tray::refresh(app);
}

#[tauri::command]
pub fn begin_shortcut_capture(app: AppHandle) -> Result<(), String> {
  diagnostics::record("capture_begin_requested", serde_json::json!({}));
  let result = (|| {
    app
      .global_shortcut()
      .unregister_all()
      .map_err(|error| error.to_string())?;
    CAPTURING.store(true, Ordering::Release);
    Ok(())
  })();
  diagnostics::record(
    "capture_begin_result",
    serde_json::json!({"error": result.as_ref().err()}),
  );
  diagnostics::snapshot(&app, "capture_begin");
  result
}

#[tauri::command]
pub fn end_shortcut_capture(app: AppHandle) -> Result<(), String> {
  diagnostics::record("capture_end_requested", serde_json::json!({}));
  let result = (|| {
    CAPTURING.store(false, Ordering::Release);
    let settings = app
      .state::<ShortcutSettingsState>()
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    for binding in settings.bindings {
      if let Some(shortcut) = binding.shortcut {
        let parsed = shortcut
          .parse::<Shortcut>()
          .map_err(|error| error.to_string())?;
        if !app.global_shortcut().is_registered(parsed) {
          register_binding(&app, binding.action, &shortcut)?;
        }
      }
    }
    Ok(())
  })();
  diagnostics::record(
    "capture_end_result",
    serde_json::json!({"error": result.as_ref().err()}),
  );
  diagnostics::snapshot(&app, "capture_end");
  result
}
