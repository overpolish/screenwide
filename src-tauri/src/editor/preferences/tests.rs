// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The look is remembered as a template. Annotations are drawn on one layer
/// of one capture, so carrying them forward would put yesterday's arrow on
/// today's screenshot - which is exactly what it did.
#[test]
fn a_remembered_look_carries_none_of_the_annotations_drawn_on_it() {
  let preferences: EditorPreferences = serde_json::from_str(
    r##"{"screenshot_output":{"annotations":[{"id":"a",
      "shape":{"kind":"arrow","start":{"x":0,"y":0},"control":{"x":1,"y":1},
      "end":{"x":2,"y":2}},"style":{"color":"#ff0000","width":6}}],
      "backgroundColor":"#171717","backgroundType":"solid",
      "backgroundRadiusPercent":0,"height":100,"width":100,
      "imageWidth":100,"radiusPercent":0,"dropShadow":true,
      "meshColors":[],"meshLockedColors":[],"meshPoints":[],"meshSeed":0,
      "meshWarpPercent":0,"meshGenerator":"mesh",
      "sourceCrop":{"x":0,"y":0,"width":1,"height":1}}}"##,
  )
  .unwrap();
  let output = preferences.screenshot_output.unwrap();
  assert_eq!(
    output.annotations.len(),
    1,
    "the fixture has an annotation on it"
  );
  assert!(screenshot_output_template(output).annotations.is_empty());
}

#[test]
fn loads_preferences_written_before_screenshot_output_was_remembered() {
  let preferences: EditorPreferences = serde_json::from_str(
    r#"{
      "screenshot_background_radius_percent": 7.5,
      "screenshot_radius_percent": 12.0
    }"#,
  )
  .unwrap();

  assert_eq!(preferences.screenshot_background_radius_percent, 7.5);
  assert_eq!(preferences.screenshot_radius_percent, 12.0);
  assert_eq!(preferences.screenshot_output, None);
  assert_eq!(preferences.recording_output, None);
  assert_eq!(
    preferences.recording_choices,
    RecordingExportChoices::default()
  );
}

/// An export only speaks for the choices it offered. A recording without a
/// camera says nothing about baking, so the next camera recording still bakes.
#[test]
fn an_export_that_did_not_offer_a_choice_keeps_the_one_remembered_before() {
  let earlier = RecordingExportChoices {
    bake_camera: Some(true),
    camera_resolution_scale_percent: Some(75),
    compression: Some(3),
    ..Default::default()
  };
  let without_camera = RecordingExportChoices {
    compression: Some(1),
    resolution_scale_ratio: Some(0.5),
    ..Default::default()
  };

  let merged = earlier.merged(without_camera);

  assert_eq!(merged.bake_camera, Some(true));
  assert_eq!(merged.camera_resolution_scale_percent, Some(75));
  assert_eq!(merged.compression, Some(1));
  assert_eq!(merged.resolution_scale_ratio, Some(0.5));
}

#[test]
fn a_damaged_file_seeds_no_choice_the_export_would_refuse() {
  let choices: RecordingExportChoices = serde_json::from_str(
    r#"{"bakeCamera":true,"cameraCompression":9,"cameraResolutionScalePercent":60,
      "compression":2,"resolutionScaleRatio":1.5}"#,
  )
  .unwrap();

  let sanitized = choices.sanitized();

  assert_eq!(sanitized.bake_camera, Some(true));
  assert_eq!(sanitized.camera_compression, None);
  assert_eq!(sanitized.camera_resolution_scale_percent, None);
  assert_eq!(sanitized.compression, Some(2));
  assert_eq!(sanitized.resolution_scale_ratio, None);
}
