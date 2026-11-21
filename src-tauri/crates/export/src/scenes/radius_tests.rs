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
/// 3% of the canvas, leaving the camera's to the recording.
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
      screen: Some(Measure::Canvas(3.0)),
      camera: None,
    }),
    inset: None,
    rotation: None,
    variant: None,
    layout_before: None,
    shapes: None,
    follow_face: None,
    cursor: None,
    background: None,
    auto: false,
  }
}

/// The screen's corners in percent of the canvas and the camera's in
/// percent of its frame `source_ms` in, for a recording whose own screen is
/// rounded by 1% of the canvas and camera drawn as a circle.
fn radii(source_ms: u64) -> (f64, f64) {
  let mut output = test_output_settings(1_600, 900);
  output.radius = Measure::Canvas(1.0);
  let mut overlay = CameraOverlaySettings {
    camera_x: 1_400.0,
    camera_y: 750.0,
    camera_width: 320.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius: Measure::Percent(50.0),
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
  let Measure::Canvas(screen) = output.radius else {
    panic!("the screen's corners are drawn as a share of the canvas");
  };
  (screen, overlay.radius_frame_percent(900.0))
}

#[test]
fn a_scene_rounds_its_panes_and_leaves_the_rest_to_the_recording() {
  let (screen, camera) = radii(5_000);
  assert!((screen - 3.0).abs() < 1e-6, "{screen} is not the scene's");
  assert!((camera - 50.0).abs() < 1e-6, "{camera} is not a circle");
  let (screen, _) = radii(12_000);
  assert!(
    (screen - 1.0).abs() < 1e-6,
    "{screen} is not the recording's"
  );
}

#[test]
fn a_scene_eases_its_radius_in_with_its_arrival() {
  let (screen, camera) = radii(1_300);
  assert!(screen > 1.0 && screen < 3.0, "{screen} is not on its way");
  assert!((camera - 50.0).abs() < 0.5, "{camera} is not a circle");
}

#[test]
fn a_radius_past_half_the_shorter_side_is_refused() {
  let mut clip = rounded_split();
  clip.radius = Some(SceneRadius {
    screen: None,
    camera: Some(Measure::Percent(60.0)),
  });
  assert!(validate_clips(&[clip]).is_err());
  assert!(validate_clips(&[rounded_split()]).is_ok());
}
