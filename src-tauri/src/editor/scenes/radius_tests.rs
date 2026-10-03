// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::model::{RecordingScenePreset, SceneRadius};
use super::*;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::test_output_settings;

/// A side by side scene from 1s to 10s that rounds the screen's corners to
/// 30%, leaving the camera's to the recording.
fn rounded_split() -> RecordingSceneClip {
  RecordingSceneClip {
    id: "r".to_owned(),
    start_ms: 1_000,
    end_ms: 10_000,
    preset: RecordingScenePreset::SplitTwoThirds,
    screen: None,
    camera: None,
    boxes: None,
    radius: Some(SceneRadius {
      screen: Some(30.0),
      camera: None,
    }),
    variant: None,
    auto: false,
  }
}

/// The screen's and the camera's radii `source_ms` in, for a recording whose
/// own screen is rounded 10% and camera 50%.
fn radii(source_ms: u64) -> (f64, f64) {
  let mut output = test_output_settings(1_600, 900);
  output.radius_percent = 10.0;
  let mut overlay = CameraOverlaySettings {
    camera_x: 1_400.0,
    camera_y: 750.0,
    camera_width: 320.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius_percent: 50.0,
  };
  arrange(
    &[rounded_split()],
    &[],
    source_ms,
    0.0,
    &mut output,
    Some((&mut overlay, (1_280, 720))),
  );
  (output.radius_percent, overlay.radius_percent)
}

#[test]
fn a_scene_rounds_its_panes_and_leaves_the_rest_to_the_recording() {
  assert_eq!(radii(5_000), (30.0, 50.0));
}

#[test]
fn a_scene_eases_its_radius_in_with_its_arrival() {
  let (screen, camera) = radii(1_300);
  assert!((screen - 20.0).abs() < 1e-6, "{screen} is not halfway");
  assert_eq!(camera, 50.0);
}

#[test]
fn a_radius_past_half_the_shorter_side_is_refused() {
  let mut clip = rounded_split();
  clip.radius = Some(SceneRadius {
    screen: None,
    camera: Some(60.0),
  });
  assert!(validate_clips(&[clip]).is_err());
  assert!(validate_clips(&[rounded_split()]).is_ok());
}
