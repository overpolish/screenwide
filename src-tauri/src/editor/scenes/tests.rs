// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::framing::SceneFraming;
use super::geometry::preset_panes;
use super::model::RecordingScenePreset;
use super::variant::SceneVariant;
use super::*;
use crate::editor::timeline_edit::TimelineRange;
use crate::editor::CameraOverlaySettings;
use crate::screenshots::{test_output_settings, ScreenshotOutputSettings};

fn clip(id: &str, start_ms: u64, end_ms: u64, preset: RecordingScenePreset) -> RecordingSceneClip {
  RecordingSceneClip {
    id: id.to_owned(),
    start_ms,
    end_ms,
    preset,
    screen: None,
    camera: None,
    boxes: None,
    radius: None,
    variant: None,
  }
}

fn split(id: &str, start_ms: u64, end_ms: u64) -> RecordingSceneClip {
  clip(id, start_ms, end_ms, RecordingScenePreset::SplitTwoThirds)
}

fn bubble(id: &str, start_ms: u64, end_ms: u64) -> RecordingSceneClip {
  clip(id, start_ms, end_ms, RecordingScenePreset::PictureInPicture)
}

/// A 1600 by 900 canvas holding a 2:1 screen, with the camera in the corner.
fn base() -> (ScreenshotOutputSettings, CameraOverlaySettings) {
  (
    test_output_settings(1_600, 900),
    CameraOverlaySettings {
      camera_x: 1_400.0,
      camera_y: 750.0,
      camera_width: 320.0,
      frame_height: 180.0,
      frame_width: 180.0,
      frame_x: 1_310.0,
      frame_y: 660.0,
      radius_percent: 50.0,
    },
  )
}

fn arranged(
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  source_ms: u64,
) -> (ScreenshotOutputSettings, CameraOverlaySettings) {
  let (mut output, mut overlay) = base();
  arrange(
    clips,
    ranges,
    source_ms,
    0.0,
    &mut output,
    Some((&mut overlay, (1_280, 720))),
  );
  (output, overlay)
}

fn zoomed(focus: (f64, f64), zoom: f64) -> RecordingSceneClip {
  RecordingSceneClip {
    screen: Some(SceneFraming {
      focus_x: focus.0,
      focus_y: focus.1,
      zoom,
    }),
    ..clip("z", 0, 10_000, RecordingScenePreset::Full)
  }
}

fn close(a: f64, b: f64) -> bool {
  (a - b).abs() < 1e-6
}

#[test]
fn clips_may_butt_but_never_overlap() {
  assert!(validate_clips(&[split("a", 0, 1_000), split("b", 1_000, 2_000)]).is_ok());
  assert!(validate_clips(&[split("a", 0, 1_000), split("b", 999, 2_000)]).is_err());
  assert!(validate_clips(&[split("b", 1_000, 2_000), split("a", 0, 1_000)]).is_err());
}

#[test]
fn clips_need_time_and_their_own_id() {
  assert!(validate_clips(&[split("a", 500, 500)]).is_err());
  assert!(validate_clips(&[split("", 0, 500)]).is_err());
  assert!(validate_clips(&[split("a", 0, 500), split("a", 600, 900)]).is_err());
}

#[test]
fn presets_use_the_webview_names() {
  let parsed: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": "a", "startMs": 0, "endMs": 10, "preset": "picture-in-picture"
  }))
  .unwrap();
  assert_eq!(parsed, bubble("a", 0, 10));
}

#[test]
fn picture_in_picture_leaves_matching_space_on_opposite_sides() {
  for canvas in [(1_920.0, 1_080.0), (1_080.0, 1_080.0), (1_080.0, 1_920.0)] {
    let panes = preset_panes(
      RecordingScenePreset::PictureInPicture,
      canvas,
      (16.0 / 10.0, 16.0 / 9.0),
      SceneVariant::default(),
    )
    .unwrap();
    let (screen, camera) = (panes.screen.unwrap(), panes.camera.unwrap());
    assert!((screen.width / screen.height - 1.6).abs() < 1e-9);
    assert!((canvas.1 - (camera.y + camera.height) - screen.y).abs() < 1e-6);
    assert!((canvas.0 - (camera.x + camera.width) - screen.x).abs() < 1e-6);
    assert!(camera.x < screen.x + screen.width && camera.y < screen.y + screen.height);
  }
}

#[test]
fn outside_every_clip_the_composition_is_untouched() {
  let clips = [split("a", 1_000, 5_000)];
  assert_eq!(arranged(&clips, &[], 500), base());
  assert_eq!(arranged(&clips, &[], 5_000), base());
}

#[test]
fn a_settled_scene_moves_the_screen_without_changing_what_it_shows() {
  let (output, overlay) = arranged(&[split("a", 0, 10_000)], &[], 5_000);
  let panes = preset_panes(
    RecordingScenePreset::SplitTwoThirds,
    (1_600.0, 900.0),
    (2.0, 16.0 / 9.0),
    SceneVariant::default(),
  )
  .unwrap();
  let (screen, camera) = (panes.screen.unwrap(), panes.camera.unwrap());
  assert!((output.crop_x - screen.x).abs() < 1e-9);
  assert!((output.crop_width - screen.width).abs() < 1e-9);
  // The test screen fills its crop, so the image is the crop.
  assert!((output.image_width - output.crop_width).abs() < 1e-9);
  assert!((output.image_y - output.crop_y).abs() < 1e-9);
  assert!((overlay.frame_x - camera.x).abs() < 1e-9);
  // The 16:9 camera fills a pane of its own shape, centred on it.
  assert!((overlay.camera_width - camera.width).abs() < 1e-9);
  assert!((overlay.camera_x - (camera.x + camera.width / 2.0)).abs() < 1e-9);
}

#[test]
fn a_scene_arrives_over_its_first_window() {
  let clips = [split("a", 1_000, 10_000)];
  let (settled, _) = arranged(&clips, &[], 5_000);
  let (halfway, _) = arranged(&clips, &[], 1_300);
  let (start, _) = base();
  let middle = (start.crop_x + settled.crop_x) / 2.0;
  assert!((halfway.crop_x - middle).abs() < 1e-6);
}

#[test]
fn butted_scenes_hand_over_without_returning_to_the_composition() {
  let clips = [split("a", 0, 5_000), bubble("b", 5_000, 10_000)];
  let (from, _) = arranged(&clips, &[], 4_999);
  let (to, _) = arranged(&clips, &[], 8_000);
  let (handing, _) = arranged(&clips, &[], 5_300);
  assert!((handing.crop_width - (from.crop_width + to.crop_width) / 2.0).abs() < 1e-6);
}

#[test]
fn a_faster_stretch_keeps_the_transition_its_length_on_screen() {
  // Played at twice the speed, 600ms of output is 1200ms of source.
  let ranges = [TimelineRange {
    output_start_us: 0,
    source_end_us: 20_000_000,
    source_start_us: 0,
    playback_rate: 2.0,
  }];
  let clips = [split("a", 0, 10_000)];
  let (settled, _) = arranged(&clips, &ranges, 5_000);
  let (halfway, _) = arranged(&clips, &ranges, 600);
  let (start, _) = base();
  assert!((halfway.crop_x - (start.crop_x + settled.crop_x) / 2.0).abs() < 1e-6);
}

#[test]
fn a_zoom_shows_less_of_the_screen_in_the_same_box() {
  let (start, _) = base();
  let (output, overlay) = arranged(&[zoomed((0.5, 0.5), 2.0)], &[], 5_000);
  // The box, and the camera outside it, stay where the recording puts them.
  assert!(close(output.crop_x, start.crop_x) && close(output.crop_width, start.crop_width));
  assert_eq!(overlay, base().1);
  assert!(close(output.image_width, start.image_width * 2.0));
  // The middle of what the crop showed is still in the middle of the box.
  let middle =
    |output: &ScreenshotOutputSettings, crop: f64| (crop - output.image_x) / output.image_width;
  let centre = start.crop_x + start.crop_width / 2.0;
  assert!(close(middle(&output, centre), middle(&start, centre)));
}

#[test]
fn a_zoom_into_a_corner_stops_where_the_crop_ends() {
  let (start, _) = base();
  let (output, _) = arranged(&[zoomed((0.0, 0.0), 2.0)], &[], 5_000);
  // What the crop showed still begins at the box's corner, never beyond it.
  assert!(close(
    output.image_x + (start.crop_x - start.image_x) * 2.0,
    output.crop_x
  ));
  assert!(close(
    output.image_y + (start.crop_y - start.image_y) * 2.0,
    output.crop_y
  ));
}

#[test]
fn annotations_grow_and_shrink_with_the_screen() {
  let (mut start, _) = base();
  start.capture_scale = 2.0;
  let in_scene = |clip: RecordingSceneClip| {
    let (mut output, mut overlay) = (start.clone(), base().1);
    arrange(
      &[clip],
      &[],
      5_000,
      0.0,
      &mut output,
      Some((&mut overlay, (1_280, 720))),
    );
    output
  };
  // Drawn at the same size against the picture, wherever the scene puts it.
  let zoom = in_scene(zoomed((0.5, 0.5), 2.0));
  assert!(close(zoom.size_scale(), 4.0));
  let shrunk = in_scene(split("a", 0, 10_000));
  assert!(close(
    shrunk.size_scale(),
    2.0 * shrunk.image_width / start.image_width
  ));
  assert!(close(shrunk.size_image_width(), start.size_image_width()));
}

#[test]
fn without_a_camera_only_full_scenes_play() {
  let (mut output, _) = base();
  let split = [split("a", 0, 10_000)];
  assert!(!arrange(&split, &[], 5_000, 0.0, &mut output, None));
  assert_eq!(output, base().0);
  assert!(arrange(
    &[zoomed((0.5, 0.5), 2.0)],
    &[],
    5_000,
    0.0,
    &mut output,
    None
  ));
  assert!(close(output.image_width, base().0.image_width * 2.0));
}

#[test]
fn framings_a_scene_cannot_hold_are_refused() {
  assert!(validate_clips(&[zoomed((0.5, 0.5), 2.0)]).is_ok());
  assert!(validate_clips(&[zoomed((0.5, 0.5), 0.5)]).is_err());
  assert!(validate_clips(&[zoomed((1.5, 0.5), 2.0)]).is_err());
}
