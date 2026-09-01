// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{validate, GlideModifier, GlidePacing, GlideSettings};

#[test]
fn an_empty_file_is_the_defaults() {
  let settings: GlideSettings = serde_json::from_str("{}").unwrap();

  assert!(settings.enabled);
  assert_eq!(settings.mouse_modifier, GlideModifier::Command);
  assert_eq!(settings.thirds_modifier, GlideModifier::Shift);
  assert_eq!(settings.window_gap, 0);
  assert!(settings.cursor_follows);
  assert!(settings.haptics);
  assert_eq!(settings.pacing, GlidePacing::Normal);
  assert!(settings.double_tap_center);
}

#[test]
fn pacing_maps_to_the_shared_detector_timings() {
  assert_eq!(GlidePacing::Snappy.rest_ms(), 40.0);
  assert_eq!(GlidePacing::Normal.rest_ms(), 60.0);
  assert_eq!(GlidePacing::Relaxed.rest_ms(), 100.0);
}

#[test]
fn reads_and_writes_the_camel_cased_names() {
  let settings: GlideSettings =
    serde_json::from_str(r#"{"windowGap":8,"doubleTapCenter":false,"pacing":"relaxed"}"#).unwrap();
  let serialized = serde_json::to_value(settings).unwrap();

  assert_eq!(settings.window_gap, 8);
  assert!(!settings.double_tap_center);
  assert_eq!(settings.pacing, GlidePacing::Relaxed);
  assert_eq!(serialized.get("windowGap").unwrap(), 8);
  assert_eq!(serialized.get("pacing").unwrap(), "relaxed");
  assert_eq!(serialized.get("cursorFollows").unwrap(), true);
}

#[test]
fn a_setting_it_does_not_know_falls_back_to_its_default() {
  let settings: GlideSettings =
    serde_json::from_str(r#"{"animationSpeed":"fast","enabled":false}"#).unwrap();

  assert!(!settings.enabled);
  assert_eq!(settings.pacing, GlidePacing::Normal);
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
    mouse_modifier: GlideModifier::Option,
    thirds_modifier: GlideModifier::Option,
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_err());
}

#[test]
fn accepts_two_modifiers_that_differ() {
  let mut settings = GlideSettings {
    mouse_modifier: GlideModifier::Control,
    thirds_modifier: GlideModifier::Option,
    ..GlideSettings::default()
  };

  assert!(validate(&mut settings).is_ok());
}
