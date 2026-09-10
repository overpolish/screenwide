// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn default_commands_match_and_disabled_ocr_does_not_intercept_keys() {
  let mut settings = OcrSettings::default();
  let runtime = Runtime::new(settings.clone()).unwrap();
  let modifier = if cfg!(target_os = "macos") { 1 } else { 2 };
  for (mac, select, copy) in [(true, 0, 8), (false, 0x41, 0x43)] {
    assert_eq!(runtime.phase(select, modifier, mac, false), Some(6));
    assert_eq!(runtime.phase(copy, modifier, mac, false), Some(7));
    assert_eq!(runtime.phase(copy, modifier, mac, true), None);
    assert_eq!(runtime.phase(copy, 0, mac, false), None);
  }
  settings.enabled = false;
  assert_eq!(
    Runtime::new(settings)
      .unwrap()
      .phase(8, modifier, true, false),
    None
  );
}

#[test]
fn remapping_and_clearing_survive_serialization() {
  let mut settings = OcrSettings::default();
  settings.bindings.insert(OcrAction::SelectAll, None);
  settings
    .bindings
    .insert(OcrAction::CopyText, Some("Shift+KeyK".to_owned()));
  let bytes = serde_json::to_vec(&settings).unwrap();
  let runtime = Runtime::new(serde_json::from_slice(&bytes).unwrap()).unwrap();
  assert_eq!(runtime.settings.bindings[&OcrAction::SelectAll], None);
  assert_eq!(runtime.phase(0x28, 8, true, false), Some(7));
  assert_eq!(runtime.phase(0x4b, 8, false, false), Some(7));
  assert_eq!(runtime.phase(0x28, 0, true, false), None);
  let modifier = if cfg!(target_os = "macos") { 1 } else { 2 };
  assert_eq!(runtime.phase(8, modifier, true, false), None);
}

#[test]
fn conflicting_and_reserved_shortcuts_are_rejected() {
  let mut settings = OcrSettings::default();
  settings.bindings.insert(
    OcrAction::CopyText,
    settings.bindings[&OcrAction::SelectAll].clone(),
  );
  assert!(Runtime::new(settings).is_err());
  assert!(shortcut::parse("Escape").is_err());
}
