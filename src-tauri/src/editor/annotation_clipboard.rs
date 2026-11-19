// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotations copied in one editor, held for a paste in any editor window.
//!
//! The editors are separate webviews, so what is copied is held here rather
//! than in one of them. It stays good for as long as nothing else has been
//! copied since: the system clipboard's change count is noted with it, and a
//! copy anywhere else, in this app or another, moves that count on and hands
//! ⌘V back to what the system clipboard holds.

use std::sync::Mutex;

use serde_json::Value;
use tauri::ipc::Invoke;

const COMMANDS: [&str; 2] = ["copy_annotations", "copied_annotations"];

/// `app_commands` with the annotation clipboard's commands beside them.
pub(crate) fn with_commands(
  app_commands: impl Fn(Invoke) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  let clipboard_commands: fn(Invoke) -> bool =
    tauri::generate_handler![copy_annotations, copied_annotations];
  move |invoke| {
    if COMMANDS.contains(&invoke.message.command()) {
      clipboard_commands(invoke)
    } else {
      app_commands(invoke)
    }
  }
}

struct Held {
  copied: Value,
  mark: u64,
}

static HELD: Mutex<Option<Held>> = Mutex::new(None);

/// Holds `copied`, the webview's own description of the annotations, until
/// something else is copied.
#[tauri::command]
pub fn copy_annotations(copied: Value) {
  *HELD.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Held {
    copied,
    mark: clipboard_mark(),
  });
}

/// The annotations copied last, or None once anything has been copied since.
#[tauri::command]
pub fn copied_annotations() -> Option<Value> {
  let held = HELD.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
  held
    .as_ref()
    .filter(|held| held.mark == clipboard_mark())
    .map(|held| held.copied.clone())
}

/// How many times the system clipboard has changed.
#[cfg(target_os = "macos")]
fn clipboard_mark() -> u64 {
  // `changeCount` may be read from any thread.
  u64::try_from(objc2_app_kit::NSPasteboard::generalPasteboard().changeCount()).unwrap_or(0)
}

#[cfg(target_os = "windows")]
fn clipboard_mark() -> u64 {
  // SAFETY: takes no arguments and only reads the clipboard's counter.
  u64::from(unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() })
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn clipboard_mark() -> u64 {
  0
}
