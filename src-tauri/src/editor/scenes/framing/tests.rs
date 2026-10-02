// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::model::RecordingScenePreset;
use super::super::{arrange, RecordingSceneClip};
use super::*;
use crate::screenshots::test_output_settings;

const ASPECT: f64 = 16.0 / 9.0;

/// A 180 point square window on a 16:9 camera zoomed to 1.5 times what covers
/// it, the face sitting right of the picture's middle.
fn recording_overlay() -> CameraOverlaySettings {
  CameraOverlaySettings {
    camera_x: 1_340.0,
    camera_y: 750.0,
    camera_width: 480.0,
    frame_height: 180.0,
    frame_width: 180.0,
    frame_x: 1_310.0,
    frame_y: 660.0,
    radius_percent: 50.0,
  }
}

const FACE: SceneFraming = SceneFraming {
  focus_x: 0.625,
  focus_y: 0.5,
  zoom: 1.5,
};

fn clip(preset: RecordingScenePreset, camera: Option<SceneFraming>) -> [RecordingSceneClip; 1] {
  [RecordingSceneClip {
    id: "a".to_owned(),
    start_ms: 0,
    end_ms: 10_000,
    preset,
    screen: None,
    camera,
    boxes: None,
    radius: None,
    variant: None,
  }]
}

/// The overlay `clips` give the camera halfway through.
fn arranged(clips: &[RecordingSceneClip]) -> CameraOverlaySettings {
  let mut output = test_output_settings(1_600, 900);
  let mut overlay = recording_overlay();
  assert!(arrange(
    clips,
    &[],
    5_000,
    0.0,
    &mut output,
    Some((&mut overlay, (1_280, 720)))
  ));
  overlay
}

fn frame(overlay: &CameraOverlaySettings) -> Rect {
  Rect {
    x: overlay.frame_x,
    y: overlay.frame_y,
    width: overlay.frame_width,
    height: overlay.frame_height,
  }
}

fn close(a: f64, b: f64) -> bool {
  (a - b).abs() < 1e-6
}

#[test]
fn a_scene_frames_the_camera_by_its_own_framing() {
  let placed = arranged(&clip(RecordingScenePreset::SplitTwoThirds, Some(FACE)));
  let shown = SceneFraming::of_camera(&placed, ASPECT);
  assert!(close(shown.zoom, FACE.zoom) && close(shown.focus_x, FACE.focus_x));
  // `recording-scene-arrangement.test.ts` pins the same numbers, so the
  // selection the webview draws lands where this frame is drawn.
  assert!(close(placed.camera_x, 1_190.625));
  assert!(close(placed.camera_y, 450.0));
  assert!(close(placed.camera_width, 675.0));
}

#[test]
fn an_unframed_arrangement_shows_the_whole_camera_from_its_middle() {
  let placed = arranged(&clip(RecordingScenePreset::SplitTwoThirds, None));
  let shown = SceneFraming::of_camera(&placed, ASPECT);
  assert!(close(shown.zoom, 1.0) && close(shown.focus_x, 0.5) && close(shown.focus_y, 0.5));
}

#[test]
fn a_full_scene_keeps_the_recordings_camera_until_it_is_reframed() {
  assert_eq!(
    arranged(&clip(RecordingScenePreset::Full, None)),
    recording_overlay()
  );
  let reframed = arranged(&clip(RecordingScenePreset::Full, Some(SceneFraming::WHOLE)));
  assert_eq!(frame(&reframed), frame(&recording_overlay()));
  assert!(close(SceneFraming::of_camera(&reframed, ASPECT).zoom, 1.0));
}

#[test]
fn a_framing_never_uncovers_its_box() {
  let frame = Rect {
    x: 100.0,
    y: 100.0,
    width: 200.0,
    height: 300.0,
  };
  let edge = SceneFraming {
    focus_x: 1.0,
    focus_y: 0.0,
    zoom: 1.0,
  };
  let (x, y, width) = edge.place(frame, ASPECT);
  let height = width / ASPECT;
  assert!(x - width / 2.0 <= frame.x + 1e-9);
  assert!(x + width / 2.0 >= frame.x + frame.width - 1e-9);
  assert!(y - height / 2.0 <= frame.y + 1e-9);
  assert!(y + height / 2.0 >= frame.y + frame.height - 1e-9);
}

#[test]
fn dragging_the_camera_moves_its_picture_with_the_pointer() {
  let before = arranged(&clip(RecordingScenePreset::SplitTwoThirds, Some(FACE)));
  let moved = FACE.reframed(frame(&before), ASPECT, (-24.0, 10.0), 1.0);
  let after = arranged(&clip(RecordingScenePreset::SplitTwoThirds, Some(moved)));
  assert!(close(after.camera_x - before.camera_x, -24.0));
  assert!(close(after.camera_y - before.camera_y, 10.0));
}

#[test]
fn a_drag_past_the_edge_stops_where_the_picture_ends() {
  let box_frame = frame(&arranged(&clip(
    RecordingScenePreset::SplitTwoThirds,
    Some(FACE),
  )));
  let far = FACE.reframed(box_frame, ASPECT, (-10_000.0, 0.0), 1.0);
  // Dragging back by any amount moves the picture at once, with no slack
  // left over from the drag that went too far.
  let back = far.reframed(box_frame, ASPECT, (10.0, 0.0), 1.0);
  assert!(back.focus_x < far.focus_x);
  assert_eq!(far, far.within(box_frame, ASPECT));
}

#[test]
fn zooming_holds_between_the_whole_picture_and_the_furthest_zoom() {
  let box_frame = frame(&recording_overlay());
  assert!(close(
    FACE.reframed(box_frame, ASPECT, (0.0, 0.0), 2.0).zoom,
    3.0
  ));
  assert!(close(
    FACE.reframed(box_frame, ASPECT, (0.0, 0.0), 0.1).zoom,
    1.0
  ));
  assert!(close(
    FACE.reframed(box_frame, ASPECT, (0.0, 0.0), 100.0).zoom,
    MAX_ZOOM
  ));
}

#[test]
fn a_box_stored_past_the_camera_is_placed_where_the_bake_draws_it() {
  // The box reaches above and left of the 16:9 picture, which the bake holds
  // inside it; the scenes and the editor's tiles start from the same place.
  let overlay = CameraOverlaySettings {
    camera_x: 400.0,
    camera_y: 300.0,
    camera_width: 480.0,
    frame_height: 200.0,
    frame_width: 300.0,
    frame_x: 100.0,
    frame_y: 120.0,
    radius_percent: 0.0,
  };
  let drawn = drawn_frame(&overlay, ASPECT);
  let baked = crate::editor::media_preview::bake_geometry(
    crate::editor::media_preview::BakedVideoExportOptions {
      camera_drop_shadow: false,
      camera_height: 1_080,
      camera_width: 1_920,
      overlay,
      screen_height: 900,
      screen_width: 1_600,
      video: crate::editor::media_preview::VideoExportOptions {
        compression: 2,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    },
  )
  .expect("a bake geometry");
  assert_eq!((drawn.x, drawn.y), (160.0, 165.0));
  assert_eq!(
    (drawn.x.round() as i32, drawn.y.round() as i32),
    (baked.frame_x, baked.frame_y)
  );
  assert_eq!(
    (drawn.width, drawn.height),
    (f64::from(baked.frame_width), f64::from(baked.frame_height))
  );
}
