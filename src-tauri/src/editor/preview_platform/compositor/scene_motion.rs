// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas rows that carry a scene's motion to the shader, which averages
//! the screen and the camera over every step they took while the shutter was
//! open.

use crate::editor::scenes::SceneMotion;
use crate::screenshots::ScreenshotOutputSettings;

/// The four rows of `Constants` a scene's motion fills: the screen's box and
/// image, each a scale, a shift in canvas pixels and, for the box, the steps
/// to average; then the camera's frame in canvas pixels and the part of its
/// picture it showed. A canvas without motion is drawn sharp, in one step.
pub(super) fn scene_motion_rows(
  settings: &ScreenshotOutputSettings,
  (width, height): (f32, f32),
) -> [[f32; 4]; 4] {
  // The motion is measured in shares of the canvas, so it fits whatever size
  // this canvas is drawn at.
  let Some(motion) = settings
    .scene_motion
    .filter(|motion| motion.samples > 1 && motion.screen[0] > 0.0 && motion.image[0] > 0.0)
  else {
    return [
      [1.0, 0.0, 0.0, 1.0],
      [1.0, 0.0, 0.0, 0.0],
      [0.0; 4],
      [0.0; 4],
    ];
  };
  let SceneMotion {
    screen,
    image,
    camera_frame,
    camera_crop,
    samples,
  } = motion;
  let [x, y, frame_width, frame_height] = camera_frame.map(|value| value as f32);
  [
    [
      screen[0] as f32,
      screen[1] as f32 * width,
      screen[2] as f32 * height,
      samples as f32,
    ],
    [
      image[0] as f32,
      image[1] as f32 * width,
      image[2] as f32 * height,
      0.0,
    ],
    [
      x * width,
      y * height,
      frame_width * width,
      frame_height * height,
    ],
    camera_crop.map(|value| value as f32),
  ]
}
