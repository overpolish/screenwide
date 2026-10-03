// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The layouts that hide a pane, take a variant or change the panes' order,
//! through the real video export: the camera alone across the canvas, a
//! stacked pair swapped so the camera is on top, and a custom scene that puts
//! the camera behind the screen.

use super::scene_video_tests::{
  blue, clip, directory, exported, frame_at, pixel, red, timeline, HEIGHT, WIDTH,
};
use super::*;
use crate::editor::CameraOverlaySettings;

fn black(pixel: [u8; 3]) -> bool {
  pixel.iter().all(|channel| *channel < 40)
}

/// A blue screen and a red camera, the camera in the recording's corner,
/// exported under `scene`.
fn export_scene(name: &str, scene: serde_json::Value) -> (PathBuf, PathBuf) {
  let directory = directory(name);
  let screen = directory.join("screen.mov");
  let camera = directory.join("camera.mov");
  clip(&screen, "color=c=blue:s=640x360:r=30:d=2");
  clip(&camera, "color=c=red:s=320x180:r=30:d=2");
  let destination = directory.join(format!("{name}.mp4"));
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
  exported(
    &screen,
    Some((&camera, options)),
    &timeline(scene),
    &destination,
  );
  (directory, destination)
}

#[test]
fn exports_the_camera_alone_across_the_canvas() {
  let (directory, destination) = export_scene(
    "camera-only",
    serde_json::json!({
      "id": "c", "startMs": 500, "endMs": 1_900, "preset": "camera-only"
    }),
  );
  // Before the scene the screen fills the canvas, the camera in its corner.
  let before = frame_at(&destination, 0.25);
  assert!(blue(pixel(&before, 320, 180)));
  assert!(red(pixel(&before, 600, 320)));
  // Settled, the camera covers the canvas edge to edge and the screen is
  // gone.
  let settled = frame_at(&destination, 1.2);
  for (x, y) in [(4, 4), (320, 180), (636, 356), (100, 300)] {
    assert!(red(pixel(&settled, x, y)), "camera at {x}, {y}");
  }
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn exports_a_stacked_pair_swapped_with_the_camera_on_top() {
  // On a 640 by 360 canvas the camera is 154 by 87 from y 40, centred over
  // a 308 by 173 screen from y 147.
  let (directory, destination) = export_scene(
    "stacked",
    serde_json::json!({
      "id": "s", "startMs": 500, "endMs": 1_900, "preset": "stacked",
      "variant": { "swap": true }
    }),
  );
  let settled = frame_at(&destination, 1.2);
  assert!(red(pixel(&settled, 320, 84)));
  assert!(blue(pixel(&settled, 320, 234)));
  // The margin around the pair, and the recording's corner the camera left.
  assert!(black(pixel(&settled, 320, 20)));
  assert!(black(pixel(&settled, 100, 180)));
  assert!(black(pixel(&settled, 600, 320)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn exports_a_custom_scene_with_the_camera_behind_the_screen() {
  // The screen fills the canvas and the camera's box sits in its middle, 128
  // by 72 from x 256, y 144: behind the screen, so hidden there.
  let (directory, destination) = export_scene(
    "camera-behind",
    serde_json::json!({
      "id": "b", "startMs": 500, "endMs": 1_900, "preset": "full",
      "boxes": {
        "screen": { "x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0 },
        "camera": { "x": 0.4, "y": 0.4, "width": 0.2, "height": 0.2 },
        "cameraBehind": true
      }
    }),
  );
  // Outside the scene the camera stays in front, in the recording's corner.
  let before = frame_at(&destination, 0.25);
  assert!(red(pixel(&before, 600, 320)));
  let settled = frame_at(&destination, 1.2);
  for (x, y) in [(320, 180), (270, 160), (600, 320)] {
    assert!(blue(pixel(&settled, x, y)), "screen at {x}, {y}");
  }
  let _ = std::fs::remove_dir_all(directory);
}
