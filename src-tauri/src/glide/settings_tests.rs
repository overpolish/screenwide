// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{validate, GlideControl, GlideSettings};
use keyboard_types::Code;

#[test]
fn an_empty_file_is_the_defaults() {
  let settings: GlideSettings = serde_json::from_str("{}").unwrap();

  assert!(settings.enabled);
  let mouse_default = if cfg!(target_os = "windows") {
    GlideControl::CONTROL
  } else {
    GlideControl::COMMAND
  };
  assert_eq!(settings.mouse_modifier, mouse_default);
  assert_eq!(settings.thirds_modifier, GlideControl::SHIFT);
  assert_eq!(settings.window_gap, 0);
  assert!(settings.cursor_follows);
  assert!(settings.haptics);
  assert!(settings.double_tap_center);
}

#[test]
fn reads_and_writes_the_camel_cased_names() {
  let settings: GlideSettings =
    serde_json::from_str(r#"{"windowGap":8,"doubleTapCenter":false}"#).unwrap();
  let serialized = serde_json::to_value(settings).unwrap();

  assert_eq!(settings.window_gap, 8);
  assert!(!settings.double_tap_center);
  assert_eq!(serialized.get("windowGap").unwrap(), 8);
  assert_eq!(serialized.get("cursorFollows").unwrap(), true);
}

/// A file written before a setting was retired - the gesture pacing, say -
/// still loads, and the retired key is simply dropped.
#[test]
fn a_setting_it_does_not_know_falls_back_to_its_default() {
  let settings: GlideSettings =
    serde_json::from_str(r#"{"pacing":"relaxed","enabled":false}"#).unwrap();

  assert!(!settings.enabled);
  assert_eq!(settings.window_gap, 0);
}

#[test]
fn clamps_a_gap_wider_than_the_grid_would_survive() {
  let mut settings = GlideSettings {
    window_gap: 900,
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_ok());
  assert_eq!(settings.window_gap, 32);
}

#[test]
fn keeps_a_gap_inside_the_range() {
  let mut settings = GlideSettings {
    window_gap: 8,
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_ok());
  assert_eq!(settings.window_gap, 8);
}

#[test]
fn refuses_one_modifier_driving_both_gestures() {
  let mut settings = GlideSettings {
    mouse_modifier: GlideControl::Key(Code::KeyQ),
    thirds_modifier: GlideControl::Key(Code::KeyQ),
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_err());
}

#[test]
fn accepts_two_modifiers_that_differ() {
  let mut settings = GlideSettings {
    mouse_modifier: GlideControl::Key(Code::KeyQ),
    thirds_modifier: GlideControl::Key(Code::KeyW),
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_ok());
}

#[test]
fn migrates_original_modifier_names_and_round_trips_any_key() {
  let old: GlideControl = serde_json::from_str(r#""control""#).unwrap();
  let letter: GlideControl = serde_json::from_str(r#""KeyQ""#).unwrap();
  assert_eq!(old, GlideControl::CONTROL);
  assert_eq!(letter, GlideControl::Key(Code::KeyQ));
  assert_eq!(serde_json::to_string(&letter).unwrap(), r#""KeyQ""#);
}

#[test]
fn round_trips_auxiliary_mouse_buttons() {
  let control: GlideControl = serde_json::from_str(r#""MouseBack""#).unwrap();
  assert_eq!(control, GlideControl::MouseBack);
  assert_eq!(serde_json::to_string(&control).unwrap(), r#""MouseBack""#);
}
