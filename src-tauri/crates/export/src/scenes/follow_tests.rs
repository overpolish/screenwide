// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera following the face: in a scene, outside every scene, and in a
//! scene that does as the recording's camera does or says otherwise.

use super::{arrange_following, camera_target};
use screenwide_output::fixtures::test_output_settings;
use screenwide_scenes::framing::drawn_frame;
use screenwide_scenes::model::RecordingScenePreset;
use screenwide_scenes::{CameraOverlaySettings, RecordingSceneClip, SceneFraming};

const ASPECT: f64 = 16.0 / 9.0;
const CAMERA: (u32, u32) = (1_280, 720);

/// A 180 point square window on a 16:9 camera zoomed to 1.5 times what
/// covers it, right of the picture's middle.
fn overlay(follow_face: bool) -> CameraOverlaySettings {
  CameraOverlaySettings {
    camera_x: 1_340.0,
    camera_y: 750.0,
    camera_width: 480.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius_percent: 50.0,
    follow_face,
  }
}

fn clip(
  preset: RecordingScenePreset,
  camera: Option<SceneFraming>,
  follow_face: Option<bool>,
) -> RecordingSceneClip {
  RecordingSceneClip {
    id: "a".to_owned(),
    start_ms: 1_000,
    end_ms: 10_000,
    preset,
    screen: None,
    camera,
    boxes: None,
    radius: None,
    rotation: None,
    variant: None,
    follow_face,
    auto: false,
  }
}

/// The camera `clips` draw `source_ms` in, from `overlay`, the face at
/// `face`.
fn camera_at(
  clips: &[RecordingSceneClip],
  overlay: CameraOverlaySettings,
  source_ms: u64,
  face: Option<(f64, f64)>,
) -> CameraOverlaySettings {
  let mut output = test_output_settings(1_600, 900);
  let mut overlay = overlay;
  arrange_following(
    clips,
    &[],
    (source_ms, 0.0),
    &mut output,
    Some((&mut overlay, CAMERA)),
    face,
  );
  overlay
}

fn shown(overlay: &CameraOverlaySettings) -> SceneFraming {
  SceneFraming::of_camera(overlay, ASPECT)
}

fn close(a: f64, b: f64) -> bool {
  (a - b).abs() < 1e-6
}

fn split(focus: (f64, f64)) -> RecordingSceneClip {
  clip(
    RecordingScenePreset::SplitTwoThirds,
    Some(SceneFraming {
      focus_x: focus.0,
      focus_y: focus.1,
      zoom: 2.0,
    }),
    Some(true),
  )
}

#[test]
fn a_scene_following_the_face_centres_it_in_the_box() {
  let placed = camera_at(
    &[split((0.5, 0.5))],
    overlay(false),
    5_000,
    Some((0.42, 0.47)),
  );
  assert!(close(shown(&placed).focus_x, 0.42) && close(shown(&placed).focus_y, 0.47));
  assert!(close(shown(&placed).zoom, 2.0));
}

#[test]
fn a_scene_following_the_face_keeps_its_offset_from_it() {
  let placed = camera_at(
    &[split((0.55, 0.45))],
    overlay(false),
    5_000,
    Some((0.42, 0.47)),
  );
  assert!(close(shown(&placed).focus_x, 0.47) && close(shown(&placed).focus_y, 0.42));
}

#[test]
fn a_face_near_the_edge_sits_as_near_the_middle_as_the_picture_allows() {
  let placed = camera_at(
    &[split((0.5, 0.5))],
    overlay(false),
    5_000,
    Some((0.5, 0.98)),
  );
  // The picture still covers the box: its bottom edge is the box's.
  let bottom = placed.camera_y + placed.camera_width / ASPECT / 2.0;
  assert!(close(bottom, placed.frame_y + placed.frame_height));
}

#[test]
fn before_the_face_is_found_a_following_scene_frames_as_it_is_set() {
  let unfound = camera_at(&[split((0.5, 0.5))], overlay(false), 5_000, None);
  assert!(close(shown(&unfound).focus_x, 0.5) && close(shown(&unfound).focus_y, 0.5));
}

#[test]
fn outside_every_scene_a_following_camera_holds_the_face_in_its_crop() {
  let still = camera_at(&[], overlay(false), 500, Some((0.4, 0.45)));
  assert_eq!(still, overlay(false));
  let followed = camera_at(&[], overlay(true), 500, Some((0.4, 0.45)));
  // The crop window stays where it is, at the zoom it gives; the face sits
  // in its middle, wherever the picture was behind it.
  assert_eq!(
    drawn_frame(&followed, ASPECT),
    drawn_frame(&overlay(true), ASPECT)
  );
  assert!(close(shown(&followed).zoom, shown(&overlay(true)).zoom));
  assert!(close(shown(&followed).focus_x, 0.4) && close(shown(&followed).focus_y, 0.45));
  // Before a scene starts too.
  let before = camera_at(&[split((0.5, 0.5))], overlay(true), 500, Some((0.4, 0.45)));
  assert_eq!(before, followed);
}

#[test]
fn a_scene_does_as_the_recordings_camera_unless_it_says_otherwise() {
  let face = Some((0.42, 0.47));
  let inheriting = [clip(RecordingScenePreset::PictureInPicture, None, None)];
  let followed = camera_at(&inheriting, overlay(true), 5_000, face);
  assert!(close(shown(&followed).focus_x, 0.42));
  let still = camera_at(&inheriting, overlay(false), 5_000, face);
  assert!(close(shown(&still).focus_x, 0.5));
  // A scene that turns following off keeps its own framing.
  let off = clip(RecordingScenePreset::PictureInPicture, None, Some(false));
  assert!(close(
    shown(&camera_at(&[off], overlay(true), 5_000, face)).focus_x,
    0.5
  ));
}

#[test]
fn a_full_scene_keeps_the_recordings_crop_followed_or_still_as_it_says() {
  let face = Some((0.42, 0.47));
  let inheriting = clip(RecordingScenePreset::Full, None, None);
  assert_eq!(
    camera_at(&[inheriting], overlay(true), 5_000, face),
    camera_at(&[], overlay(true), 500, face),
  );
  let off = clip(RecordingScenePreset::Full, None, Some(false));
  let still = camera_at(&[off], overlay(true), 5_000, face);
  assert_eq!((still.camera_x, still.camera_y), (1_340.0, 750.0));
  let on = clip(RecordingScenePreset::Full, None, Some(true));
  let followed = camera_at(&[on], overlay(false), 5_000, face);
  assert!(close(shown(&followed).focus_x, 0.42));
  assert!(close(shown(&followed).zoom, shown(&overlay(false)).zoom));
}

#[test]
fn a_drag_on_a_following_full_scene_starts_from_the_face_in_the_middle() {
  let output = test_output_settings(1_600, 900);
  let full = [clip(RecordingScenePreset::Full, None, None)];
  let (_, _, framing) =
    camera_target(&full, 5_000, &output, &overlay(true), CAMERA).expect("the camera is placed");
  assert!(close(framing.focus_x, 0.5) && close(framing.focus_y, 0.5));
  assert!(close(framing.zoom, shown(&overlay(true)).zoom));
}
