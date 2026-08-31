// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::{is_recording_ui_visible, WindowLabel};

static ARMED: AtomicBool = AtomicBool::new(false);
const ESCAPE: &str = "Escape";
const DISMISS_REQUESTED_EVENT: &str = "recording-ui://dismiss-requested";
const SCREENSHOT_DISMISS_REQUESTED_EVENT: &str = "screenshot-region://dismiss-requested";
const RULER_DISMISS_REQUESTED_EVENT: &str = "ruler://dismiss-requested";
const TEXT_RECOGNITION_DISMISS_REQUESTED_EVENT: &str = "text-recognition://dismiss-requested";

const fn should_be_armed(
  controls_visible: bool,
  screenshot_session: bool,
  ruler_active: bool,
  text_recognition_active: bool,
) -> bool {
  text_recognition_active || screenshot_session || ruler_active || controls_visible
}

/// A capture overlay borrows Escape while it is active. Otherwise the visible
/// recording bar owns it even though its panel does not take focus.
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
      if event.state() == ShortcutState::Pressed {
        if crate::text_recognition::is_active(app) {
          let _ = app.emit_to(
            WindowLabel::RecordingBar.as_str(),
            TEXT_RECOGNITION_DISMISS_REQUESTED_EVENT,
            (),
          );
          return;
        }
        if crate::ruler::is_active(app) {
          let _ = app.emit_to(
            WindowLabel::RecordingBar.as_str(),
            RULER_DISMISS_REQUESTED_EVENT,
            (),
          );
          return;
        }
        if super::region::SCREENSHOT_REGION_SESSION.load(Ordering::Acquire) {
          let _ = app.emit_to(
            WindowLabel::RegionSelector.as_str(),
            SCREENSHOT_DISMISS_REQUESTED_EVENT,
            (),
          );
          return;
        }
        if !is_recording_ui_visible() {
          return;
        }
        if super::source_selector::is_expanded() {
          let _ = super::source_selector::collapse(app.clone(), Some(true));
          return;
        }
        if super::options::is_standalone_listbox_open() {
          let _ = super::options::close_standalone_listbox(app.clone(), true);
          return;
        }
        if super::options::is_recording_options_open() {
          let _ = super::options::close_recording_options(app.clone(), true);
          return;
        }
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

/// Gives Escape to exactly one owner. Native desktop panels use the global
/// route because non-activating peer surfaces cannot reliably receive keys.
pub(super) fn sync(
  app: &AppHandle,
  controls_visible: bool,
  screenshot_session: bool,
  ruler_active: bool,
) {
  if should_be_armed(
    controls_visible,
    screenshot_session,
    ruler_active,
    crate::text_recognition::is_active(app),
  ) {
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
    assert!(should_be_armed(true, true, false, false));
    assert!(should_be_armed(false, true, true, false));
    assert!(should_be_armed(true, false, false, false));
    assert!(!should_be_armed(false, false, false, false));
  }

  #[test]
  fn text_recognition_borrows_escape_without_recording_controls() {
    assert!(should_be_armed(false, false, false, true));
  }

  #[test]
  fn ruler_borrows_escape_from_visible_recording_ui() {
    assert!(should_be_armed(true, false, true, false));
    assert!(should_be_armed(false, false, true, false));
  }
}
