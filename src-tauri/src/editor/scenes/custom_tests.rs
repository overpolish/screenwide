// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::model::{RecordingScenePreset, SceneBox, SceneBoxes};
use super::*;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::test_output_settings;

const SCREEN: SceneBox = SceneBox {
  x: 0.1,
  y: 0.1,
  width: 0.5,
  height: 0.5,
};

const CAMERA: SceneBox = SceneBox {
  x: 0.7,
  y: 0.6,
  width: 0.2,
  height: 0.3,
};

fn custom(preset: RecordingScenePreset, camera: Option<SceneBox>) -> RecordingSceneClip {
  RecordingSceneClip {
    id: "c".to_owned(),
    start_ms: 0,
    end_ms: 10_000,
    preset,
    screen: None,
    camera: None,
    boxes: Some(SceneBoxes {
      screen: SCREEN,
      camera,
      camera_behind: false,
    }),
    radius: None,
    variant: None,
    auto: false,
  }
}

fn overlay() -> CameraOverlaySettings {
  CameraOverlaySettings {
    camera_x: 1_400.0,
    camera_y: 750.0,
    camera_width: 320.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius_percent: 50.0,
  }
}

#[test]
fn a_custom_scene_puts_the_panes_in_its_boxes_not_its_presets() {
  let mut output = test_output_settings(1_600, 900);
  let mut camera = overlay();
  assert!(arrange(
    &[custom(RecordingScenePreset::SplitTwoThirds, Some(CAMERA))],
    &[],
    5_000,
    0.0,
    &mut output,
    Some((&mut camera, (1_280, 720))),
  ));
  assert_eq!(
    (
      output.crop_x,
      output.crop_y,
      output.crop_width,
      output.crop_height
    ),
    (160.0, 90.0, 800.0, 450.0)
  );
  assert_eq!(
    (
      camera.frame_x.round(),
      camera.frame_y.round(),
      camera.frame_width.round(),
      camera.frame_height.round()
    ),
    (1_120.0, 540.0, 320.0, 270.0)
  );
}

#[test]
fn a_custom_scene_without_a_camera_box_plays_without_a_camera() {
  let mut output = test_output_settings(1_600, 900);
  // Its preset places a camera, but the boxes the scene was made custom from
  // had none to place.
  assert!(arrange(
    &[custom(RecordingScenePreset::PictureInPicture, None)],
    &[],
    5_000,
    0.0,
    &mut output,
    None,
  ));
  assert_eq!(output.crop_width, 800.0);
  assert!(!arrange(
    &[custom(RecordingScenePreset::Full, Some(CAMERA))],
    &[],
    5_000,
    0.0,
    &mut test_output_settings(1_600, 900),
    None,
  ));
}

#[test]
fn a_scene_putting_the_camera_behind_crosses_the_order_over_as_it_arrives() {
  let front = RecordingSceneClip {
    end_ms: 5_000,
    ..custom(RecordingScenePreset::Full, Some(CAMERA))
  };
  let mut behind: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "b", "startMs": 5_000, "endMs": 10_000, "preset": "full",
    "boxes": {
      "screen": { "x": 0.1, "y": 0.1, "width": 0.5, "height": 0.5 },
      "camera": { "x": 0.7, "y": 0.6, "width": 0.2, "height": 0.3 },
      "cameraBehind": true
    }
  }))
  .unwrap();
  assert!(behind.boxes.is_some_and(|boxes| boxes.camera_behind));
  let order = |clips: &[RecordingSceneClip], source_ms: u64| {
    let mut output = test_output_settings(1_600, 900);
    arrange(
      clips,
      &[],
      source_ms,
      0.0,
      &mut output,
      Some((&mut overlay(), (1_280, 720))),
    );
    output.scene_camera_front
  };
  let clips = [front, behind.clone()];
  assert_eq!(order(&clips, 2_000), Some(1.0));
  // Halfway through the hand-over the two orders are mixed evenly.
  assert!(order(&clips, 5_300).is_some_and(|front| (front - 0.5).abs() < 1e-6));
  assert_eq!(order(&clips, 8_000), Some(0.0));
  // A scene that leaves the order alone keeps the camera in front.
  behind.boxes = behind.boxes.map(|boxes| SceneBoxes {
    camera_behind: false,
    ..boxes
  });
  assert_eq!(order(&[behind], 8_000), Some(1.0));
}

#[test]
fn a_box_dragged_away_keeps_its_middle_on_the_canvas() {
  let dragged = SCREEN.moved((2.0, -2.0), 1.0);
  assert_eq!((dragged.x, dragged.y), (0.75, -0.25));
  let shrunk = CAMERA.moved((0.0, 0.0), 0.001);
  assert!((shrunk.width - 0.02).abs() < 1e-9);
  assert!((shrunk.height / shrunk.width - 1.5).abs() < 1e-9);
}

#[test]
fn boxes_whose_middle_leaves_the_canvas_are_refused() {
  assert!(validate_clips(&[custom(RecordingScenePreset::Full, None)]).is_ok());
  let mut lost = custom(RecordingScenePreset::Full, None);
  lost.boxes = Some(SceneBoxes {
    screen: SceneBox { x: 0.9, ..SCREEN },
    camera: None,
    camera_behind: false,
  });
  assert!(validate_clips(&[lost]).is_err());
}
