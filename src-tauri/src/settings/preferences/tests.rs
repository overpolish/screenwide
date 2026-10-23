// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::GeneralSettings;

#[test]
fn opens_export_location_when_the_setting_is_missing() {
  let settings: GeneralSettings = serde_json::from_str("{}").unwrap();

  assert!(settings.open_location_after_export);
}

#[test]
fn preserves_an_explicitly_disabled_export_location() {
  let settings: GeneralSettings =
    serde_json::from_str(r#"{"openLocationAfterExport":false}"#).unwrap();

  assert!(!settings.open_location_after_export);
}

#[test]
fn starts_with_no_saved_backgrounds() {
  let settings: GeneralSettings = serde_json::from_str("{}").unwrap();

  assert!(settings.background_presets.is_empty());
}

#[test]
fn keeps_a_saved_background_preset() {
  let settings: GeneralSettings = serde_json::from_str(
    r##"{"backgroundPresets":[{"id":"a","name":"Slate","background":{"kind":"solid","color":"#333333"}}]}"##,
  )
  .unwrap();
  let serialized = serde_json::to_value(&settings).unwrap();

  assert_eq!(settings.background_presets.len(), 1);
  assert_eq!(settings.background_presets[0].name, "Slate");
  assert_eq!(
    serialized["backgroundPresets"][0]["background"]["kind"],
    "solid"
  );
}

#[test]
fn accepts_but_does_not_reserialize_the_retired_capture_on_draw_setting() {
  let settings: GeneralSettings =
    serde_json::from_str(r#"{"captureScreenshotOnDraw":false}"#).unwrap();
  let serialized = serde_json::to_value(settings).unwrap();

  assert!(serialized.get("captureScreenshotOnDraw").is_none());
}

#[test]
fn keeps_launching_without_the_controls_as_saved_for_the_recording_bar() {
  let settings: GeneralSettings =
    serde_json::from_str(r#"{"showRecordingBarOnLaunch":false}"#).unwrap();

  assert!(!settings.show_recording_controls_on_launch);
}
