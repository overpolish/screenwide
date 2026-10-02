// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A custom scene through the real video export: its own boxes, a screen box
//! cropped to a shape of its own, and the scene's own corner radius.

use super::scene_video_tests::{
  blue, clip, directory, exported, frame_at, pixel, red, timeline, HEIGHT, WIDTH,
};
use super::*;
use crate::editor::CameraOverlaySettings;

fn black(pixel: [u8; 3]) -> bool {
  pixel.iter().all(|channel| *channel < 40)
}

#[test]
fn exports_a_custom_scene_in_its_own_boxes_and_corners() {
  let directory = directory("custom");
  let screen = directory.join("screen.mov");
  let camera = directory.join("camera.mov");
  // The screen's left half is red and its right half blue.
  clip(
    &screen,
    "color=c=blue:s=640x360:r=30:d=2,drawbox=x=0:y=0:w=320:h=360:color=red:t=fill",
  );
  clip(&camera, "color=c=red:s=320x180:r=30:d=2");
  // From half a second to 1.9s, settled from 1.1s to 1.3s: the screen in a
  // 160 by 180 box from x 320, narrower than the screen, so it shows the
  // screen's middle third and the line between its halves at x 400; its
  // corners rounded to half its shorter side, a semicircle across its top;
  // the camera in a 128 by 108 box from x 32, y 36.
  let timeline = timeline(serde_json::json!({
    "id": "c", "startMs": 500, "endMs": 1_900, "preset": "full",
    "boxes": {
      "screen": { "x": 0.5, "y": 0.25, "width": 0.25, "height": 0.5 },
      "camera": { "x": 0.05, "y": 0.1, "width": 0.2, "height": 0.3 }
    },
    "radius": { "screen": 50.0 }
  }));
  let destination = directory.join("custom.mp4");
  let options = BakedVideoExportOptions {
    camera_drop_shadow: false,
    camera_height: 180,
    camera_width: 320,
    overlay: CameraOverlaySettings {
      camera_width: 80.0,
      camera_x: 600.0,
      camera_y: 320.0,
      frame_height: 45.0,
      frame_width: 80.0,
      frame_x: 560.0,
      frame_y: 297.5,
      radius_percent: 0.0,
    },
    screen_height: HEIGHT,
    screen_width: WIDTH,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  };
  exported(&screen, Some((&camera, options)), &timeline, &destination);

  // Before the scene the screen fills the canvas.
  let before = frame_at(&destination, 0.25);
  assert!(red(pixel(&before, 20, 20)));
  assert!(blue(pixel(&before, 400, 180)));

  let settled = frame_at(&destination, 1.2);
  // The screen box shows the screen's middle, cut at its own shape.
  assert!(red(pixel(&settled, 360, 200)));
  assert!(blue(pixel(&settled, 440, 200)));
  assert!(black(pixel(&settled, 300, 200)));
  assert!(black(pixel(&settled, 500, 200)));
  // Its top is the semicircle its radius makes: inside near the middle, the
  // canvas showing through at the box's own corner.
  assert!(red(pixel(&settled, 380, 100)));
  assert!(black(pixel(&settled, 326, 96)));
  // The camera sits in its own box, and no longer in the recording's corner.
  assert!(red(pixel(&settled, 96, 90)));
  assert!(black(pixel(&settled, 20, 20)));
  assert!(black(pixel(&settled, 600, 320)));
  let _ = std::fs::remove_dir_all(directory);
}
