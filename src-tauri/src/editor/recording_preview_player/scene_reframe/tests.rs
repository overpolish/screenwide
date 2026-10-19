// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn the_whole_camera_overhangs_a_square_box_from_its_middle() {
  // The editor's `reframeWindow` pins the same picture for the same box.
  let mut overlay = CameraOverlaySettings {
    camera_x: 190.0,
    camera_y: 190.0,
    camera_width: 640.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 100.0,
    frame_y: 100.0,
    radius_percent: 50.0,
    follow_face: false,
  };
  show_whole_camera(&mut overlay, (1_920, 1_080));
  assert_eq!(
    (
      overlay.frame_x,
      overlay.frame_y,
      overlay.frame_width,
      overlay.frame_height
    ),
    (30.0, 100.0, 320.0, 180.0)
  );
  assert_eq!(
    (overlay.camera_x, overlay.camera_y, overlay.camera_width),
    (190.0, 190.0, 320.0)
  );
  assert_eq!(overlay.radius_percent, 0.0);
}

#[test]
fn a_custom_screen_cropped_in_place_shows_its_picture_where_it_sits() {
  // A 2:1 screen in a box at a tenth of a 1600 by 900 canvas, zoomed twice
  // past covering it. The editor's `pictureRect` pins the same rect.
  let clip: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "c",
    "startMs": 0,
    "endMs": 10_000,
    "preset": "full",
    "screen": { "focusX": 0.5, "focusY": 0.5, "zoom": 2.0 },
    "boxes": { "screen": { "x": 0.1, "y": 0.1, "width": 0.5, "height": 0.5 } },
  }))
  .expect("a custom clip");
  assert!(crops_in_place(&clip, SceneReframe::Screen));
  assert!(!crops_in_place(&clip, SceneReframe::Camera));
  assert_eq!(
    reframed_clips(
      std::slice::from_ref(&clip),
      Some(SceneReframe::Screen),
      5_000
    ),
    None
  );
  let base = screenwide_output::fixtures::test_output_settings(1_600, 900);
  let mut output = base.clone();
  assert!(screenwide_export::scenes::arrange(
    &[clip],
    &[],
    5_000,
    0.0,
    &mut output,
    None
  ));
  show_screen_picture(&mut output, &base);
  let shown = [
    output.crop_x,
    output.crop_y,
    output.crop_width,
    output.crop_height,
  ];
  for (shown, expected) in shown.into_iter().zip([-340.0, -135.0, 1_800.0, 900.0]) {
    assert!((shown - expected).abs() < 1e-6, "{shown} is not {expected}");
  }
}

#[test]
fn the_scenes_turn_of_the_screen_eases_and_a_reframed_screen_stands_upright() {
  let clip: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "t",
    "startMs": 0,
    "endMs": 10_000,
    "preset": "full",
    "rotation": { "x": 20.0, "y": -30.0, "z": 10.0 },
  }))
  .expect("a turned clip");
  let turned = |crop: SceneCrop| {
    let mut output = screenwide_output::fixtures::test_output_settings(1_600, 900);
    assert!(screenwide_export::scenes::arrange(
      std::slice::from_ref(&clip),
      &[],
      5_000,
      0.0,
      &mut output,
      None
    ));
    ease_screen_turn(&mut output, crop.screen_turn());
    output.rotation
  };
  let halfway = SceneCrop {
    pane: None,
    screen_turn: 0.5,
  };
  assert_eq!(
    turned(halfway),
    Some(SceneRotation {
      x: 10.0,
      y: -15.0,
      z: 5.0
    })
  );
  let reframing = SceneCrop {
    pane: Some(SceneReframe::Screen),
    screen_turn: 1.0,
  };
  assert_eq!(turned(reframing), None);
  assert!(turned(SceneCrop::default()).is_some_and(|rotation| rotation.x == 20.0));
}
