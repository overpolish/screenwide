// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::fixtures::test_output_settings;
use screenwide_scenes::model::{RecordingScenePreset, SceneRadius};
use screenwide_scenes::validate_clips;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_scenes::Measure;
use screenwide_scenes::RecordingSceneClip;

/// A side by side scene from 1s to 10s that rounds the screen's corners by
/// 24 points, leaving the camera's to the recording.
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
      screen: Some(Measure::Points(24.0)),
      camera: None,
    }),
    inset: None,
    rotation: None,
    variant: None,
    follow_face: None,
    cursor: None,
    background: None,
    auto: false,
  }
}

/// The screen's corners in points and the camera's in percent `source_ms`
/// in, for a recording captured at two pixels to the point whose own screen
/// is rounded 10 points and camera 50%.
fn radii(source_ms: u64) -> (f64, f64) {
  let mut output = test_output_settings(1_600, 900);
  output.capture_width_points = output.image_width / 2.0;
  output.radius = Measure::Points(10.0);
  let mut overlay = CameraOverlaySettings {
    camera_x: 1_400.0,
    camera_y: 750.0,
    camera_width: 320.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius_percent: 50.0,
    follow_face: false,
  };
  arrange(
    &[rounded_split()],
    &[],
    source_ms,
    0.0,
    &mut output,
    Some((&mut overlay, (1_280, 720))),
  );
  let Measure::Points(screen) = output.radius else {
    panic!("the screen's corners are drawn in points");
  };
  (screen, overlay.radius_percent)
}

#[test]
fn a_scene_rounds_its_panes_and_leaves_the_rest_to_the_recording() {
  let (screen, camera) = radii(5_000);
  assert!((screen - 24.0).abs() < 1e-6, "{screen} is not the scene's");
  assert_eq!(camera, 50.0);
  let (screen, _) = radii(12_000);
  assert!(
    (screen - 10.0).abs() < 1e-6,
    "{screen} is not the recording's"
  );
}

#[test]
fn a_scene_eases_its_radius_in_with_its_arrival() {
  let (screen, camera) = radii(1_300);
  assert!(screen > 10.0 && screen < 24.0, "{screen} is not on its way");
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
