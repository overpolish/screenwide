// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn defaults_and_saved_settings_round_trip_without_losing_unbound_actions() {
  let mut settings = RulerSettings::default();
  settings.bindings.insert(RulerAction::CopyColour, None);
  let bytes = serde_json::to_vec(&settings).unwrap();
  let runtime = Runtime::new(serde_json::from_slice(&bytes).unwrap()).unwrap();
  assert_eq!(runtime.settings.bindings[&RulerAction::CopyColour], None);
  assert_eq!(runtime.settings.bindings.len(), 13);
  assert!(resolve(&runtime, 48, 0, true, false, false).is_none());
}

#[test]
fn remapped_held_tool_resolves_on_both_platforms_and_ignores_repeats() {
  let mut settings = RulerSettings::default();
  settings
    .bindings
    .insert(RulerAction::StampHorizontal, Some("Shift+KeyA".into()));
  let runtime = Runtime::new(settings).unwrap();
  for (key, mac) in [(0, true), (0x41, false)] {
    assert_eq!(
      resolve(&runtime, key, 8, mac, false, false),
      Some(KeyCommand {
        phase: 20,
        release: Some(22)
      })
    );
    assert!(resolve(&runtime, key, 0, mac, false, false).is_none());
    assert!(resolve(&runtime, key, 8, mac, true, false).is_none());
    assert!(resolve(&runtime, key, 8, mac, false, true).is_none());
  }
  assert!(resolve(&runtime, 18, 0, true, false, false).is_none());
}

#[test]
fn disabled_ruler_does_not_resolve_commands() {
  let runtime = Runtime::new(RulerSettings {
    enabled: false,
    ..Default::default()
  })
  .unwrap();
  assert!(resolve(&runtime, 7, 0, true, false, false).is_none());
}

#[test]
fn invalid_and_conflicting_bindings_are_rejected() {
  for value in [
    "Escape",
    "ShiftLeft",
    "MouseMiddle",
    "Bogus+KeyA",
    "Control+Control+KeyA",
    "KeyX",
  ] {
    let mut settings = RulerSettings::default();
    settings
      .bindings
      .insert(RulerAction::CycleTolerance, Some(value.into()));
    assert!(Runtime::new(settings).is_err(), "accepted {value}");
  }
}

#[test]
fn modifier_aliases_cannot_create_duplicate_shortcuts() {
  let mut settings = RulerSettings::default();
  let primary = if cfg!(target_os = "macos") {
    "Super"
  } else {
    "Control"
  };
  settings
    .bindings
    .insert(RulerAction::CycleTolerance, Some(format!("{primary}+KeyC")));
  assert!(Runtime::new(settings).is_err());
}

#[test]
fn legacy_alternatives_remain_until_the_action_is_rebound() {
  let mut settings = RulerSettings::default();
  let runtime = Runtime::new(settings.clone()).unwrap();
  let command = if cfg!(target_os = "macos") { 1 } else { 2 };
  assert_eq!(
    resolve(&runtime, 0x75, 0, true, false, false)
      .unwrap()
      .phase,
    16
  );
  assert_eq!(
    resolve(&runtime, 0x59, command, false, false, false)
      .unwrap()
      .phase,
    19
  );
  settings
    .bindings
    .insert(RulerAction::DeleteMeasurement, Some("KeyD".into()));
  settings.bindings.insert(RulerAction::Redo, None);
  let runtime = Runtime::new(settings).unwrap();
  assert!(resolve(&runtime, 0x75, 0, true, false, false).is_none());
  assert!(resolve(&runtime, 0x59, command, false, false, false).is_none());
}

#[test]
fn missing_actions_receive_defaults_without_replacing_existing_bindings() {
  let settings: RulerSettings =
    serde_json::from_str(r#"{"enabled":false,"bindings":{"cycleTolerance":"KeyK"}}"#).unwrap();
  let runtime = Runtime::new(settings).unwrap();
  assert_eq!(runtime.settings.bindings.len(), 13);
  assert_eq!(
    runtime.settings.bindings[&RulerAction::CycleTolerance].as_deref(),
    Some("KeyK")
  );
  assert!(!runtime.settings.enabled);
}
