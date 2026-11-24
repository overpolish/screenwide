// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn a_centred_grow_moves_placement_by_half_the_growth() {
  let old = WorldRect {
    x: 100.0,
    y: 100.0,
    width: 3600.0,
    height: 2338.0,
  };
  let new = WorldRect {
    x: 100.0 - (6765.0 - 3600.0) / 2.0,
    y: 100.0 - (4894.0 - 2338.0) / 2.0,
    width: 6765.0,
    height: 4894.0,
  };
  let shift = origin_shift(old, new);
  let mut output = screenwide_output::fixtures::test_output_settings(3600, 2338);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_x = 0.0;
  output.image_y = 0.0;
  shift_output(&mut output, shift);
  // The layer stays where it was on screen: half the growth now sits
  // before it on each axis.
  assert_eq!(output.crop_x, (6765.0 - 3600.0) / 2.0);
  assert_eq!(output.crop_y, (4894.0 - 2338.0) / 2.0);
  assert_eq!(output.image_x, output.crop_x);
  assert_eq!(output.image_y, output.crop_y);
}

#[test]
fn a_far_edge_grow_leaves_placement_alone() {
  let old = WorldRect {
    x: 0.0,
    y: 0.0,
    width: 100.0,
    height: 100.0,
  };
  let new = WorldRect {
    x: 0.0,
    y: 0.0,
    width: 150.0,
    height: 100.0,
  };
  let mut output = screenwide_output::fixtures::test_output_settings(100, 100);
  output.crop_x = 10.0;
  output.image_x = 5.0;
  shift_output(&mut output, origin_shift(old, new));
  assert_eq!(output.crop_x, 10.0);
  assert_eq!(output.image_x, 5.0);
}

/// A camera of no consequence: the canvas tested holds the screen alone.
fn camera() -> CameraOverlaySettings {
  CameraOverlaySettings {
    camera_x: 0.0,
    camera_y: 0.0,
    camera_width: 64.0,
    frame_height: 64.0,
    frame_width: 64.0,
    frame_x: 0.0,
    frame_y: 0.0,
    radius: screenwide_scenes::Measure::Percent(0.0),
    follow_face: false,
  }
}

/// An 8252 by 1800 canvas holding a 2359 wide picture near its left, as a
/// wide main canvas does, and the recording scene drawn from it.
fn wide_canvas() -> (WorkspaceScene, RecordingOutputSettings) {
  let mut primary = screenwide_output::fixtures::test_output_settings(8252, 1800);
  primary.crop_x = 373.0;
  primary.image_x = 373.0;
  primary.crop_y = 140.0;
  primary.image_y = 140.0;
  primary.crop_width = 2359.0;
  primary.image_width = 2359.0;
  primary.crop_height = 1476.0;
  let output = RecordingOutputSettings {
    camera: screenwide_output::fixtures::test_output_settings(64, 64),
    primary,
  };
  let pane = WorldRect {
    x: 0.0,
    y: 0.0,
    width: 8252.0,
    height: 1800.0,
  };
  let scene = recording_scene(
    pane,
    &[WorkspacePane { id: 0, rect: pane }],
    false,
    camera(),
    &output,
    0,
  )
  .unwrap();
  (scene, output)
}

#[test]
fn a_canvas_narrowed_past_its_picture_stops_where_the_picture_is_in_reach() {
  use screenwide_compositor::workspace_editor::{FRAME_EDGE_CENTERED, FRAME_EDGE_RIGHT};
  let (scene, output) = wide_canvas();
  let edges = FRAME_EDGE_RIGHT | FRAME_EDGE_CENTERED;
  let resize = |share: f64| {
    resize_recording_frame(&scene, &output, camera(), 0, edges, (share, 0.0))
      .unwrap()
      .1
      .primary
  };
  // Dragged to nothing, it stops at the narrowest canvas still holding the
  // picture, which the export takes as a placement.
  let narrowest = resize(-1.0);
  assert!(narrowest.width > 64, "{}", narrowest.width);
  assert!(
    screenwide_output::output_placement(2359, 1476, &narrowest).is_ok(),
    "{narrowest:?}"
  );
  // Anything wider than that is where the drag has it: centred, both sides
  // come in by the drag, a fifth of the canvas each.
  assert_eq!(resize(-0.2).width, 4951);
  // And dragged back out in the same gesture, it is as it was.
  assert_eq!(resize(0.0).width, 8252);
  assert_eq!(resize(0.0).image_x, 373.0);
}
