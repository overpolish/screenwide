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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EscapeOwner {
  Annotate,
  TextRecognition,
  Screenshot,
  Ruler,
  RecordingControls,
}

const fn owner(
  controls_visible: bool,
  screenshot_session: bool,
  ruler_active: bool,
  text_recognition_active: bool,
  annotate_active: bool,
) -> Option<EscapeOwner> {
  if annotate_active {
    // Nothing can be up alongside the annotate overlay: starting it dismisses
    // every other capture tool, and each of theirs dismisses it.
    Some(EscapeOwner::Annotate)
  } else if text_recognition_active {
    Some(EscapeOwner::TextRecognition)
  } else if screenshot_session {
    // Quick Screenshot borrows the Region surface while preserving Ruler
    // underneath it. Escape must return that borrowed surface first; Ruler
    // becomes the owner again after the screenshot session has ended.
    Some(EscapeOwner::Screenshot)
  } else if ruler_active {
    Some(EscapeOwner::Ruler)
  } else if controls_visible {
    Some(EscapeOwner::RecordingControls)
  } else {
    None
  }
}

const fn should_be_armed(
  controls_visible: bool,
  screenshot_session: bool,
  ruler_active: bool,
  text_recognition_active: bool,
  annotate_active: bool,
) -> bool {
  owner(
    controls_visible,
    screenshot_session,
    ruler_active,
    text_recognition_active,
    annotate_active,
  )
  .is_some()
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
        crate::shortcuts::during_native_callback(|| dismiss_owner(app));
      }
    })
    .is_err()
  {
    ARMED.store(false, Ordering::Release);
  }
}

/// Hands one Escape press to whoever owns it.
fn dismiss_owner(app: &AppHandle) {
  let screenshot_session = super::region::SCREENSHOT_REGION_SESSION.load(Ordering::Acquire);
  match owner(
    is_recording_ui_visible(),
    screenshot_session,
    crate::ruler::is_active(app),
    crate::text_recognition::is_active(app),
    crate::annotate::is_active(app),
  ) {
    Some(EscapeOwner::Annotate) => {
      // Carried out here rather than through the recording bar: the overlay is
      // native, so there is no frontend turn to wait for.
      crate::annotate::dismiss(app);
      return;
    }
    Some(EscapeOwner::TextRecognition) => {
      let _ = app.emit_to(
        WindowLabel::RecordingBar.as_str(),
        TEXT_RECOGNITION_DISMISS_REQUESTED_EVENT,
        (),
      );
      return;
    }
    Some(EscapeOwner::Screenshot) => {
      let _ = app.emit_to(
        WindowLabel::RegionSelector.as_str(),
        SCREENSHOT_DISMISS_REQUESTED_EVENT,
        (),
      );
      return;
    }
    Some(EscapeOwner::Ruler) => {
      let _ = app.emit_to(
        WindowLabel::RecordingBar.as_str(),
        RULER_DISMISS_REQUESTED_EVENT,
        (),
      );
      return;
    }
    Some(EscapeOwner::RecordingControls) => {}
    None => return,
  }
  if !is_recording_ui_visible() {
    return;
  }
  if super::source_selector::is_expanded() {
    let _ = super::source_selector::collapse(app.clone(), Some(true));
    return;
  }
  if super::options::is_standalone_listbox_open() {
    let _ = super::options::close_all_standalone_listboxes(app.clone(), true);
    return;
  }
  // Teardown runs on the bar's later IPC turn rather than unregistering the
  // shortcut from inside its native callback.
  let _ = app.emit_to(
    WindowLabel::RecordingBar.as_str(),
    DISMISS_REQUESTED_EVENT,
    (),
  );
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
  // Capture tools are dismissed from inside native shortcut callbacks, and the
  // shortcut registry is locked for the length of one. Escape changes hands on
  // the next turn instead; whatever was taken down is already down by then.
  if crate::shortcuts::in_native_callback() {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      sync(&app, controls_visible, screenshot_session, ruler_active);
    });
    return;
  }
  if should_be_armed(
    controls_visible,
    screenshot_session,
    ruler_active,
    crate::text_recognition::is_active(app),
    crate::annotate::is_active(app),
  ) {
    arm(app);
  } else {
    disarm(app);
  }
}

#[cfg(test)]
mod tests {
  use super::{owner, should_be_armed, EscapeOwner};

  #[test]
  fn screenshot_session_borrows_escape_from_visible_recording_ui() {
    assert!(should_be_armed(true, true, false, false, false));
    assert!(should_be_armed(false, true, true, false, false));
    assert!(should_be_armed(true, false, false, false, false));
    assert!(!should_be_armed(false, false, false, false, false));
  }

  #[test]
  fn text_recognition_borrows_escape_without_recording_controls() {
    assert!(should_be_armed(false, false, false, true, false));
  }

  #[test]
  fn ruler_borrows_escape_from_visible_recording_ui() {
    assert!(should_be_armed(true, false, true, false, false));
    assert!(should_be_armed(false, false, true, false, false));
  }

  #[test]
  fn screenshot_borrows_escape_before_preserved_ruler() {
    assert_eq!(
      owner(true, true, true, false, false),
      Some(EscapeOwner::Screenshot)
    );
    assert_eq!(
      owner(true, false, true, false, false),
      Some(EscapeOwner::Ruler)
    );
  }

  #[test]
  fn the_annotate_overlay_owns_escape_while_it_is_up() {
    assert!(should_be_armed(false, false, false, false, true));
    assert_eq!(
      owner(true, true, true, true, true),
      Some(EscapeOwner::Annotate)
    );
  }
}
