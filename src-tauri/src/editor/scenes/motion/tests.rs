// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{arrange, RecordingSceneClip};
use crate::editor::scenes::model::RecordingScenePreset;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::{test_output_settings, ScreenshotOutputSettings};

const FRAME_MS: f32 = 1_000.0 / 60.0;

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

/// The screen as a split arriving at 1s has it `source_ms` in, over a frame
/// covering `frame_ms`.
fn screen(source_ms: u64, frame_ms: f32) -> ScreenshotOutputSettings {
  let clips = [RecordingSceneClip {
    id: "a".to_owned(),
    start_ms: 1_000,
    end_ms: 10_000,
    preset: RecordingScenePreset::SplitTwoThirds,
    screen: None,
    camera: None,
    boxes: None,
    radius: None,
    variant: None,
  }];
  let mut output = test_output_settings(1_600, 900);
  arrange(
    &clips,
    &[],
    source_ms,
    frame_ms,
    &mut output,
    Some((&mut overlay(), (1_280, 720))),
  );
  output
}

#[test]
fn a_moving_frame_carries_where_the_screen_was_when_its_shutter_opened() {
  let drawn = screen(1_300, FRAME_MS);
  let opened = screen(1_283, 0.0);
  let motion = drawn.scene_motion.expect("the screen moved over the frame");
  let [scale, shift_x, shift_y] = motion.screen;
  // Carried by the motion, the drawn screen lands where the shutter found it.
  assert!((drawn.crop_x * scale + shift_x * 1_600.0 - opened.crop_x).abs() < 1e-6);
  assert!((drawn.crop_y * scale + shift_y * 900.0 - opened.crop_y).abs() < 1e-6);
  assert!((drawn.crop_width * scale - opened.crop_width).abs() < 1e-6);
  assert!(motion.samples > 1);
}

#[test]
fn a_settled_scene_and_a_still_draw_sharp() {
  assert_eq!(screen(5_000, FRAME_MS).scene_motion, None);
  assert_eq!(screen(1_300, 0.0).scene_motion, None);
}

#[test]
fn a_frame_opening_before_a_scene_blurs_from_the_composition_outside_it() {
  // A frame covering 150ms, its shutter opening before the clip begins.
  let drawn = screen(1_100, 150.0);
  let start = test_output_settings(1_600, 900);
  let motion = drawn.scene_motion.expect("the screen moved over the frame");
  let [scale, shift_x, _] = motion.screen;
  assert!((drawn.crop_x * scale + shift_x * 1_600.0 - start.crop_x).abs() < 1e-6);
}

#[test]
fn a_zoom_blurs_the_screen_image_inside_a_still_box() {
  let zoom: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "z", "startMs": 1_000, "endMs": 10_000, "preset": "full",
    "screen": { "focusX": 0.5, "focusY": 0.5, "zoom": 2.0 }
  }))
  .unwrap();
  let mut output = test_output_settings(1_600, 900);
  arrange(&[zoom], &[], 1_300, FRAME_MS, &mut output, None);
  let motion = output
    .scene_motion
    .expect("the zoom moved the image over the frame");
  // The box stays where the recording puts it while the image grows in it.
  assert_eq!(motion.screen, [1.0, 0.0, 0.0]);
  assert!(motion.image[0] < 1.0);
}
