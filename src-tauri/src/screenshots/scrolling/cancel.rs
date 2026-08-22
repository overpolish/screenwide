// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Only one scrolling capture can run at a time — the screenshot workspace is
/// reserved for the whole of it — so the request needs no per-job identity.
static REQUESTED: AtomicBool = AtomicBool::new(false);

const ESCAPE: &str = "Escape";

/// Escape is claimed for the length of the capture rather than handled by the
/// overlay, which is click-through and never focused: the window the capture is
/// driving keeps key focus throughout, so nothing else could hear the key.
///
/// Returns whether the key was actually claimed, so the overlay only offers a
/// way out that exists.
pub(super) fn arm(app: &AppHandle) -> bool {
  REQUESTED.store(false, Ordering::Release);
  ESCAPE.parse::<Shortcut>().is_ok_and(|shortcut| {
    app
      .global_shortcut()
      .on_shortcut(shortcut, move |_, _, event| {
        if event.state() == ShortcutState::Pressed {
          REQUESTED.store(true, Ordering::Release);
        }
      })
      .is_ok()
  })
}

pub(super) fn disarm(app: &AppHandle) {
  if let Ok(shortcut) = ESCAPE.parse::<Shortcut>() {
    let _ = app.global_shortcut().unregister(shortcut);
  }
  REQUESTED.store(false, Ordering::Release);
}

pub(super) fn is_requested() -> bool {
  REQUESTED.load(Ordering::Acquire)
}

/// The capture unwinds through the ordinary error path, so the message it
/// carries is never shown; this is what tells the command the stop was asked
/// for rather than a failure.
pub(super) fn was_requested() -> bool {
  REQUESTED.swap(false, Ordering::AcqRel)
}
