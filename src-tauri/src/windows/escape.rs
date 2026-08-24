// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::{is_recording_ui_visible, WindowLabel};

static ARMED: AtomicBool = AtomicBool::new(false);
const ESCAPE: &str = "Escape";
const DISMISS_REQUESTED_EVENT: &str = "recording-ui://dismiss-requested";

const fn should_be_armed(controls_visible: bool, screenshot_session: bool) -> bool {
  controls_visible && !screenshot_session
}

/// The screenshot overlay borrows Escape while its one-shot capture is open.
/// Otherwise the visible recording bar owns it even though its panel does not
/// take focus.
pub(super) fn arm(app: &AppHandle) {
  if ARMED
    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
    .is_err()
  {
    return;
  }
  let Ok(shortcut) = ESCAPE.parse::<Shortcut>() else {
    ARMED.store(false, Ordering::Release);
    return;
  };
  if app
    .global_shortcut()
    .on_shortcut(shortcut, |app, _, event| {
      if event.state() == ShortcutState::Pressed && is_recording_ui_visible() {
        // Teardown runs on the bar's later IPC turn rather than unregistering
        // the shortcut from inside its native callback.
        let _ = app.emit_to(
          WindowLabel::RecordingBar.as_str(),
          DISMISS_REQUESTED_EVENT,
          (),
        );
      }
    })
    .is_err()
  {
    ARMED.store(false, Ordering::Release);
  }
}

pub(super) fn disarm(app: &AppHandle) {
  if !ARMED.swap(false, Ordering::AcqRel) {
    return;
  }
  if let Ok(shortcut) = ESCAPE.parse::<Shortcut>() {
    let _ = app.global_shortcut().unregister(shortcut);
  }
}

/// Gives Escape to exactly one owner. A screenshot overlay needs the ordinary
/// key event in its WebView, so the recording UI's global shortcut must be
/// absent for the entire screenshot session rather than merely ignoring its
/// callback.
pub(super) fn sync(app: &AppHandle, controls_visible: bool, screenshot_session: bool) {
  if should_be_armed(controls_visible, screenshot_session) {
    arm(app);
  } else {
    disarm(app);
  }
}

#[cfg(test)]
mod tests {
  use super::should_be_armed;

  #[test]
  fn screenshot_session_borrows_escape_from_visible_recording_ui() {
    assert!(!should_be_armed(true, true));
    assert!(should_be_armed(true, false));
    assert!(!should_be_armed(false, false));
  }
}
