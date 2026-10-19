// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::fixtures::test_output_settings;
use screenwide_scenes::model::RecordingScenePreset;
use screenwide_scenes::{validate_clips, RecordingSceneClip, SceneRotation};

const TILTED: SceneRotation = SceneRotation {
  x: 20.0,
  y: -30.0,
  z: 10.0,
};

fn turned(id: &str, start_ms: u64, end_ms: u64, rotation: SceneRotation) -> RecordingSceneClip {
  RecordingSceneClip {
    id: id.to_owned(),
    start_ms,
    end_ms,
    preset: RecordingScenePreset::Full,
    screen: None,
    camera: None,
    boxes: None,
    radius: None,
    rotation: Some(rotation),
    variant: None,
    follow_face: None,
    auto: false,
  }
}

/// How `clips` turn a screen turned `own` on its own, `source_ms` in.
fn rotation_over(
  own: Option<SceneRotation>,
  clips: &[RecordingSceneClip],
  source_ms: u64,
) -> Option<SceneRotation> {
  let mut output = test_output_settings(1_600, 900);
  output.rotation = own;
  arrange(clips, &[], source_ms, 0.0, &mut output, None);
  output.rotation
}

/// How `clips` turn an upright screen `source_ms` in.
fn rotation(clips: &[RecordingSceneClip], source_ms: u64) -> Option<SceneRotation> {
  rotation_over(None, clips, source_ms)
}

#[test]
fn a_settled_scene_turns_the_screen_as_it_holds() {
  let clips = [turned("a", 1_000, 10_000, TILTED)];
  assert_eq!(rotation(&clips, 5_000), Some(TILTED));
  // Outside every scene the screen stands upright.
  assert_eq!(rotation(&clips, 500), None);
}

#[test]
fn a_scene_turns_the_screen_in_with_its_arrival() {
  let clips = [turned("a", 1_000, 10_000, TILTED)];
  let halfway = rotation(&clips, 1_300).expect("the screen is turning in");
  assert!((halfway.x - 10.0).abs() < 1e-9);
  assert!((halfway.y + 15.0).abs() < 1e-9);
  assert!((halfway.z - 5.0).abs() < 1e-9);
}

#[test]
fn a_scene_replaces_the_screens_own_turn_and_arrives_from_it() {
  let upright = SceneRotation::UPRIGHT;
  let clips = [turned("a", 1_000, 10_000, upright)];
  // Outside every scene the screen keeps its own turn.
  assert_eq!(rotation_over(Some(TILTED), &clips, 500), Some(TILTED));
  // Settled, a scene that stands the screen upright draws it upright.
  assert_eq!(rotation_over(Some(TILTED), &clips, 5_000), None);
  // Arriving, the turn runs from the screen's own to the scene's.
  let halfway = rotation_over(Some(TILTED), &clips, 1_300).expect("the screen is turning in");
  assert!((halfway.x - 10.0).abs() < 1e-9);
  assert!((halfway.y + 15.0).abs() < 1e-9);
  assert!((halfway.z - 5.0).abs() < 1e-9);
}

#[test]
fn butted_scenes_hand_one_turn_on_to_the_next() {
  let spun = SceneRotation {
    x: 0.0,
    y: 0.0,
    z: 370.0,
  };
  let clips = [
    turned("a", 1_000, 5_000, TILTED),
    turned("b", 5_000, 10_000, spun),
  ];
  let between = rotation(&clips, 5_300).expect("the screen is turning on");
  assert!((between.x - 10.0).abs() < 1e-9);
  // Each angle runs on its own, so a turn past a whole one spins round.
  assert!((between.z - 190.0).abs() < 1e-9);
}

#[test]
fn a_turn_is_held_to_ten_whole_turns_either_way() {
  let valid = |y: f64| {
    validate_clips(&[turned(
      "a",
      1_000,
      10_000,
      SceneRotation { x: 0.0, y, z: 0.0 },
    )])
    .is_ok()
  };
  assert!(valid(-3_600.0) && valid(3_600.0));
  assert!(!valid(3_600.5) && !valid(f64::NAN) && !valid(f64::INFINITY));
}
