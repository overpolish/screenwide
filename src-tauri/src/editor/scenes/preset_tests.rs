// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{preset_panes, PresetPanes, Rect};
use super::model::RecordingScenePreset;
use super::variant::{SceneBubbleSize, SceneCameraSize, SceneCorner, SceneVariant};
use super::*;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::{test_output_settings, ScreenshotOutputSettings};

const CANVAS: (f64, f64) = (1_600.0, 900.0);

/// The panes on a 1600 by 900 canvas for a 2:1 screen and a 16:9 camera.
fn panes(preset: RecordingScenePreset, variant: SceneVariant) -> (Rect, Rect) {
  let PresetPanes { screen, camera } =
    preset_panes(preset, CANVAS, (2.0, 16.0 / 9.0), variant).unwrap();
  (screen.unwrap(), camera.unwrap())
}

fn assert_rect(rect: Rect, (x, y, width, height): (f64, f64, f64, f64)) {
  for (got, want) in [
    (rect.x, x),
    (rect.y, y),
    (rect.width, width),
    (rect.height, height),
  ] {
    assert!((got - want).abs() < 0.01, "{rect:?} against {want}");
  }
}

fn clip(preset: RecordingScenePreset) -> RecordingSceneClip {
  RecordingSceneClip {
    id: "p".to_owned(),
    start_ms: 1_000,
    end_ms: 10_000,
    preset,
    screen: None,
    camera: None,
    boxes: None,
    radius: None,
    variant: None,
    auto: false,
  }
}

/// The recording's own composition: a 2:1 screen across 80% of a 1600 by
/// 900 canvas, with a 180 point camera in its corner.
fn arranged(
  clip: RecordingSceneClip,
  source_ms: u64,
) -> (ScreenshotOutputSettings, CameraOverlaySettings) {
  let mut output = test_output_settings(1_600, 900);
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
    &[clip],
    &[],
    source_ms,
    0.0,
    &mut output,
    Some((&mut overlay, (1_280, 720))),
  );
  (output, overlay)
}

#[test]
fn stacked_keeps_each_pane_at_its_own_shape_centred_on_the_other() {
  let (screen, camera) = panes(RecordingScenePreset::Stacked, SceneVariant::default());
  assert!((camera.height - screen.height / 2.0).abs() < 1e-9);
  assert!((camera.width / camera.height - 16.0 / 9.0).abs() < 1e-9);
  assert!(camera.y > screen.y + screen.height);
  // The pair is centred, the space above matching the space below.
  assert!((screen.y - (CANVAS.1 - (camera.y + camera.height))).abs() < 1e-9);
  // Swapped at two thirds the camera, twice the screen's height, is on top
  // and whole; `recording-scene-presets.test.ts` pins the same numbers.
  let (screen, camera) = panes(
    RecordingScenePreset::Stacked,
    SceneVariant {
      swap: true,
      camera_size: Some(SceneCameraSize::TwoThirds),
      ..SceneVariant::default()
    },
  );
  assert_rect(camera, (414.81, 100.0, 770.37, 433.33));
  assert_rect(screen, (583.33, 583.33, 433.33, 216.67));
}

#[test]
fn side_by_side_can_give_the_camera_two_thirds_on_the_left_at_its_own_shape() {
  let (screen, camera) = panes(
    RecordingScenePreset::SplitTwoThirds,
    SceneVariant {
      swap: true,
      camera_size: Some(SceneCameraSize::TwoThirds),
      ..SceneVariant::default()
    },
  );
  assert_rect(camera, (100.0, 196.88, 900.0, 506.25));
  assert_rect(screen, (1_050.0, 337.5, 450.0, 225.0));
}

#[test]
fn a_small_picture_in_picture_sits_over_the_corner_it_names() {
  let (screen, camera) = panes(
    RecordingScenePreset::PictureInPicture,
    SceneVariant {
      corner: Some(SceneCorner::TopLeft),
      size: Some(SceneBubbleSize::Small),
      ..SceneVariant::default()
    },
  );
  assert!((camera.width - screen.height * 0.4).abs() < 1e-9);
  assert!(camera.x < screen.x && camera.y < screen.y);
  assert!(camera.x + camera.width > screen.x && camera.y + camera.height > screen.y);
}

#[test]
fn camera_only_fills_the_canvas_and_hides_the_screen() {
  let (output, overlay) = arranged(clip(RecordingScenePreset::CameraOnly), 5_000);
  assert_eq!(output.scene_opacity, Some([0.0, 1.0]));
  assert_eq!(
    (
      overlay.frame_x,
      overlay.frame_y,
      overlay.frame_width,
      overlay.frame_height
    ),
    (0.0, 0.0, 1_600.0, 900.0)
  );
  // The canvas's own corners cut it, not a radius of its own.
  assert_eq!(overlay.radius_percent, 0.0);
}

#[test]
fn screen_only_keeps_the_screen_and_hides_the_camera() {
  let (output, _) = arranged(clip(RecordingScenePreset::ScreenOnly), 5_000);
  let base = test_output_settings(1_600, 900);
  assert_eq!(output.scene_opacity, Some([1.0, 0.0]));
  assert_eq!(
    (output.crop_x, output.crop_width),
    (base.crop_x, base.crop_width)
  );
}

#[test]
fn a_camera_a_scene_hides_shrinks_where_it_was_as_it_fades() {
  // Halfway through the screen-only scene's arrival the camera is half
  // faded and five eighths of its size, still centred where it was drawn.
  let (output, overlay) = arranged(clip(RecordingScenePreset::ScreenOnly), 1_300);
  assert_eq!(output.scene_opacity, Some([1.0, 0.5]));
  assert!((overlay.frame_width - 180.0 * 0.625).abs() < 1e-9);
  assert!((overlay.frame_x + overlay.frame_width / 2.0 - 1_400.0).abs() < 1e-9);
  assert!((overlay.frame_y + overlay.frame_height / 2.0 - 750.0).abs() < 1e-9);
}
