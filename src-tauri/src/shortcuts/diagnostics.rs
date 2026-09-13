// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bounded local history for failures that disappear when the app restarts.

use serde_json::{json, Value};
use std::{
  io::Write,
  path::Path,
  sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::{sync_channel, SyncSender},
    OnceLock,
  },
};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

static SENDER: OnceLock<SyncSender<Value>> = OnceLock::new();
static DROPPED: AtomicU64 = AtomicU64::new(0);
const MAX_BYTES: u64 = 256 * 1024;
const LOG_NAME: &str = "shortcuts.jsonl";

pub(crate) fn initialize(app: &AppHandle) {
  let directory = match app.path().app_log_dir() {
    Ok(path) => path,
    Err(error) => {
      eprintln!("Shortcut diagnostics unavailable: {error}");
      return;
    }
  };
  let (sender, receiver) = sync_channel(256);
  if SENDER.set(sender).is_err() {
    return;
  }
  let spawn = std::thread::Builder::new()
    .name("shortcut-diagnostics".into())
    .spawn(move || {
      for event in receiver {
        if let Err(error) = append(&directory, &event, MAX_BYTES) {
          eprintln!("Could not write shortcut diagnostics: {error}");
        }
      }
    });
  if let Err(error) = spawn {
    eprintln!("Could not start shortcut diagnostics: {error}");
  }
  record(
    "process_started",
    json!({"version": env!("CARGO_PKG_VERSION")}),
  );
}

/// Never wait for disk or for queue space from a global-shortcut callback.
/// Only recognized actions/settings are logged; ordinary keystrokes are not.
pub(crate) fn record(event: &str, details: Value) {
  let Some(sender) = SENDER.get() else {
    return;
  };
  let dropped = DROPPED.swap(0, Ordering::Relaxed);
  let entry = json!({
    "time": chrono::Utc::now().to_rfc3339(), "pid": std::process::id(),
    "event": event, "details": details, "droppedSincePrevious": dropped,
  });
  if sender.try_send(entry).is_err() {
    DROPPED.fetch_add(dropped + 1, Ordering::Relaxed);
  }
}

fn append(directory: &Path, entry: &Value, limit: u64) -> std::io::Result<()> {
  std::fs::create_dir_all(directory)?;
  let path = directory.join(LOG_NAME);
  let mut line = serde_json::to_vec(entry)?;
  line.push(b'\n');
  if std::fs::metadata(&path).is_ok_and(|meta| meta.len() + line.len() as u64 > limit) {
    let previous = directory.join("shortcuts.previous.jsonl");
    match std::fs::remove_file(&previous) {
      Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
      _ => {}
    }
    std::fs::rename(&path, previous)?;
  }
  std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)?
    .write_all(&line)
}

pub(super) fn snapshot(app: &AppHandle, reason: &str) {
  let settings = app
    .state::<super::ShortcutSettingsState>()
    .0
    .lock()
    .unwrap_or_else(|error| error.into_inner())
    .clone();
  let bindings = settings.bindings.iter().map(|binding| {
    let parsed = binding.shortcut.as_deref().map(str::parse::<Shortcut>);
    json!({
      "action": binding.action, "shortcut": binding.shortcut,
      "enabled": super::feature_availability::action_enabled(binding.action),
      // This is the plugin's bookkeeping, not a query of macOS hotkey ownership.
      "pluginRegistered": parsed.as_ref().and_then(|p| p.as_ref().ok()).map(|p| app.global_shortcut().is_registered(*p)),
      "parseError": parsed.as_ref().and_then(|p| p.as_ref().err()).map(ToString::to_string),
    })
  }).collect::<Vec<_>>();
  record(
    "registration_snapshot",
    json!({
      "reason": reason, "capturing": super::is_capturing(), "bindings": bindings,
      "pendingRecording": crate::editor::has_pending_workspace_kind(app, crate::editor::EditorKind::Recording),
      "pendingScreenshot": crate::editor::has_pending_workspace_kind(app, crate::editor::EditorKind::Screenshot),
    }),
  );
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FrontendPhase {
  ListenerReady,
  ListenerStopped,
  ListenerFailed,
  Received,
  DuplicateIgnored,
  Completed,
  Failed,
}

/// Persist acknowledgements so event delivery can be distinguished from a
/// registered shortcut that never reached the screenshot frontend.
#[tauri::command]
pub fn report_shortcut_diagnostic(
  window: tauri::WebviewWindow,
  phase: FrontendPhase,
  action: Option<super::ShortcutAction>,
  error: Option<String>,
) {
  record(
    "screenshot_frontend",
    json!({
      "window": window.label(), "phase": phase, "action": action,
      "error": error.map(|text| text.chars().take(2000).collect::<String>()),
    }),
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn appends_across_restarts_and_retains_one_rotated_file() {
    let directory = std::env::temp_dir().join(format!(
      "shortcut-log-{}-{}",
      std::process::id(),
      chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    let first = json!({"event": "first"});
    let second = json!({"event": "second"});
    append(&directory, &first, 1000).unwrap();
    append(&directory, &second, 1000).unwrap();
    let lines = std::fs::read_to_string(directory.join(LOG_NAME)).unwrap();
    assert_eq!(lines.lines().count(), 2);
    append(&directory, &json!({"event": "third"}), 1).unwrap();
    assert_eq!(
      std::fs::read_to_string(directory.join("shortcuts.previous.jsonl")).unwrap(),
      lines
    );
    append(&directory, &json!({"event": "fourth"}), 1).unwrap();
    let previous = std::fs::read_to_string(directory.join("shortcuts.previous.jsonl")).unwrap();
    assert_eq!(
      serde_json::from_str::<Value>(&previous).unwrap()["event"],
      "third"
    );
    std::fs::remove_dir_all(directory).unwrap();
  }
}
