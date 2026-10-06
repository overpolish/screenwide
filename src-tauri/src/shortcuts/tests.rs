// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use super::*;

#[test]
fn defaults_open_the_bar_save_replays_take_screenshots_and_recognize_text() {
  let settings = ShortcutSettings::default();
  let assigned = settings
    .bindings
    .iter()
    .filter(|binding| binding.shortcut.is_some())
    .map(|binding| (binding.action, binding.shortcut.as_deref().unwrap()))
    .collect::<Vec<_>>();
  assert_eq!(
    assigned,
    [
      (
        ShortcutAction::ToggleRecordingBar,
        "CommandOrControl+Shift+Digit6"
      ),
      (ShortcutAction::SaveReplay, "CommandOrControl+Shift+Digit9"),
      (
        ShortcutAction::TakeScreenshot,
        "CommandOrControl+Shift+Digit8"
      ),
      (ShortcutAction::RecognizeText, "CommandOrControl+Shift+KeyT"),
      (ShortcutAction::RulerOverlay, "CommandOrControl+Shift+KeyR"),
      (
        ShortcutAction::AnnotateOverlay,
        "CommandOrControl+Shift+KeyA"
      ),
      (
        ShortcutAction::AnnotateClear,
        "CommandOrControl+Shift+Backspace"
      ),
    ]
  );
  assert!(settings
    .bindings
    .iter()
    .find(|binding| binding.action == ShortcutAction::TakeScreenshotToClipboard)
    .is_some_and(|binding| binding.shortcut.is_none()));
}

#[test]
fn a_saved_file_that_predates_an_action_leaves_it_on_its_default() {
  let stored = ShortcutSettings {
    bindings: vec![
      ShortcutBinding {
        action: ShortcutAction::RecognizeText,
        shortcut: None,
      },
      ShortcutBinding {
        action: ShortcutAction::RulerOverlay,
        shortcut: Some("Command+Shift+KeyR".to_owned()),
      },
    ],
  };
  let merged = merge(Some(stored));
  let shortcut = |action| {
    merged
      .bindings
      .iter()
      .find(|binding| binding.action == action)
      .and_then(|binding| binding.shortcut.clone())
  };

  // Never saved, so the default it shipped with stands.
  assert_eq!(
    shortcut(ShortcutAction::AnnotateOverlay).as_deref(),
    Some("CommandOrControl+Shift+KeyA")
  );
  // Saved as cleared, so it stays cleared.
  assert_eq!(shortcut(ShortcutAction::RecognizeText), None);
  assert_eq!(
    shortcut(ShortcutAction::RulerOverlay).as_deref(),
    Some("Command+Shift+KeyR")
  );
}

#[test]
fn a_new_default_on_keys_the_user_already_bound_is_left_unbound() {
  let stored = ShortcutSettings {
    bindings: vec![ShortcutBinding {
      action: ShortcutAction::TakeScreenshotToClipboard,
      shortcut: Some("Super+Shift+Digit9".to_owned()),
    }],
  };
  let merged = merge(Some(stored));
  let shortcut = |action| {
    merged
      .bindings
      .iter()
      .find(|binding| binding.action == action)
      .and_then(|binding| binding.shortcut.clone())
  };

  // Save Replay's default is the same keys spelled another way.
  assert_eq!(shortcut(ShortcutAction::SaveReplay), None);
  assert_eq!(
    shortcut(ShortcutAction::TakeScreenshotToClipboard).as_deref(),
    Some("Super+Shift+Digit9")
  );
  // Other new defaults on free keys still arrive.
  assert_eq!(
    shortcut(ShortcutAction::AnnotateOverlay).as_deref(),
    Some("CommandOrControl+Shift+KeyA")
  );
}

#[test]
fn each_frontend_action_goes_to_the_window_that_performs_it() {
  assert_eq!(
    action_window(ShortcutAction::ToggleRecordingBar).map(WindowLabel::as_str),
    Some(WindowLabel::RecordingBar.as_str())
  );
  assert_eq!(
    action_window(ShortcutAction::StartStopRecording).map(WindowLabel::as_str),
    Some(WindowLabel::RecordingBar.as_str())
  );
  assert_eq!(
    action_window(ShortcutAction::TakeScreenshot).map(WindowLabel::as_str),
    Some(WindowLabel::RegionSelector.as_str())
  );
  assert_eq!(
    action_window(ShortcutAction::TakeScreenshotToClipboard).map(WindowLabel::as_str),
    Some(WindowLabel::RegionSelector.as_str())
  );
  assert_eq!(
    action_window(ShortcutAction::RecognizeText).map(WindowLabel::as_str),
    Some(WindowLabel::RecordingBar.as_str())
  );
}

#[test]
fn taking_a_screenshot_never_reaches_the_recording_bar() {
  assert_ne!(
    action_window(ShortcutAction::TakeScreenshot).map(WindowLabel::as_str),
    action_window(ShortcutAction::StartStopRecording).map(WindowLabel::as_str)
  );
}

#[test]
fn capture_window_graphs_are_never_changed_inside_shortcut_callbacks() {
  for action in [
    ShortcutAction::TakeScreenshot,
    ShortcutAction::TakeScreenshotToClipboard,
    ShortcutAction::RecognizeText,
  ] {
    assert!(requires_frontend_turn(action));
  }
}

#[test]
fn taking_a_screenshot_keeps_the_ruler_and_the_live_annotations() {
  use crate::capture_overlays::CaptureOverlay;
  for action in [
    ShortcutAction::TakeScreenshot,
    ShortcutAction::TakeScreenshotToClipboard,
  ] {
    let preserved = preserved_capture_overlays(action);
    assert!(preserved.contains(&CaptureOverlay::Ruler));
    assert!(preserved.contains(&CaptureOverlay::Annotate));
    assert!(!preserved.contains(&CaptureOverlay::TextRecognition));
  }
  assert_eq!(
    preserved_capture_overlays(ShortcutAction::RulerOverlay),
    &[CaptureOverlay::Ruler]
  );
  assert_eq!(
    preserved_capture_overlays(ShortcutAction::RecognizeText),
    &[CaptureOverlay::TextRecognition]
  );
}

#[test]
fn the_actions_rust_handles_alone_ask_no_window() {
  for action in [
    ShortcutAction::PauseResumeRecording,
    ShortcutAction::RulerOverlay,
  ] {
    assert!(action_window(action).is_none());
  }
}

#[test]
fn every_action_appears_once() {
  let settings = ShortcutSettings::default();
  let actions = settings
    .bindings
    .iter()
    .map(|binding| binding.action)
    .collect::<HashSet<_>>();
  assert_eq!(actions.len(), settings.bindings.len());
}
