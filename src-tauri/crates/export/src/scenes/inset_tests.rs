// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_output::fixtures::test_output_settings;
use screenwide_output::{output_placement, ScreenshotOutputSettings};
use screenwide_scenes::validate_clips;
use screenwide_scenes::RecordingSceneClip;

/// A full scene from 1s to 10s that zooms twice into the screen's middle,
/// padded by `inset` of its box's shorter side.
fn padded(inset: f64) -> RecordingSceneClip {
  serde_json::from_value(serde_json::json!({
    "id": "p", "startMs": 1_000, "endMs": 10_000, "preset": "full",
    "screen": { "focusX": 0.5, "focusY": 0.5, "zoom": 2.0 },
    "inset": inset
  }))
  .unwrap()
}

fn arranged(clip: RecordingSceneClip, source_ms: u64) -> ScreenshotOutputSettings {
  let mut output = test_output_settings(1_600, 900);
  arrange(&[clip], &[], source_ms, 0.0, &mut output, None);
  output
}

fn close(a: f64, b: f64) -> bool {
  (a - b).abs() < 1e-6
}

#[test]
fn a_padded_scene_grows_its_box_outwards_and_shows_the_picture_inside_it() {
  let unpadded = arranged(padded(0.0), 5_000);
  let output = arranged(padded(0.1), 5_000);
  let pad = 0.1 * unpadded.crop_width.min(unpadded.crop_height);
  assert!(close(output.crop_x, unpadded.crop_x - pad));
  assert!(close(output.crop_y, unpadded.crop_y - pad));
  assert!(close(output.crop_width, unpadded.crop_width + pad * 2.0));
  assert!(close(output.crop_height, unpadded.crop_height + pad * 2.0));
  // The picture stays where the scene puts it, shown only in the box.
  assert_eq!(
    (output.image_x, output.image_y, output.image_width),
    (unpadded.image_x, unpadded.image_y, unpadded.image_width)
  );
  assert_eq!(
    output.scene_window,
    Some([
      unpadded.crop_x,
      unpadded.crop_y,
      unpadded.crop_width,
      unpadded.crop_height
    ])
  );
  assert_eq!(unpadded.scene_window, None);
  // The compositor draws the picture in the box and the inset colour round it.
  let (width, height) = (1_600, 800);
  let placement = output_placement(width, height, &output).unwrap();
  assert_eq!(
    (
      placement.source_crop_x,
      placement.source_crop_y,
      placement.source_crop_width,
      placement.source_crop_height
    ),
    (
      unpadded.crop_x.round() as i32,
      unpadded.crop_y.round() as i32,
      unpadded.crop_width.round() as u32,
      unpadded.crop_height.round() as u32
    )
  );
}

#[test]
fn a_scene_eases_its_padding_in_with_its_arrival() {
  let settled = arranged(padded(0.1), 5_000);
  let halfway = arranged(padded(0.1), 1_300);
  let window = halfway.scene_window.expect("the scene is padding its box");
  let pad = (
    window[0] - halfway.crop_x,
    settled.scene_window.unwrap()[0] - settled.crop_x,
  );
  assert!(
    close(pad.0 * 2.0, pad.1),
    "{} is not half of {}",
    pad.0,
    pad.1
  );
}

#[test]
fn outside_the_scene_the_recording_keeps_its_own_padding() {
  let outside = arranged(padded(0.1), 500);
  assert_eq!(outside, test_output_settings(1_600, 900));
}

#[test]
fn a_padding_past_half_the_shorter_side_is_refused() {
  assert!(validate_clips(&[padded(0.5)]).is_ok());
  assert!(validate_clips(&[padded(0.6)]).is_err());
  assert!(validate_clips(&[padded(-0.1)]).is_err());
}
