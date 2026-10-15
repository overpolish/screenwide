// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas rows that turn a layer in depth, by its own turn or by the one
//! a scene gives it. The shader reads every pixel the layer's pane covers
//! back through the inverse of the turn, to the point of the upright pane it
//! shows, and draws that point as it always has: the picture, its crop,
//! rounding and shadow, its annotations and the cursor all turn together.
//! What lies over the canvas as drawn, the camera and the blur layers, is
//! found for a point of the pane through the turn itself. While a scene moves
//! the screen, each step the frame averages is read back through a blend of
//! the turn the shutter opened on and the frame's own.

use screenwide_output::{OutputPlacement, ScreenshotOutputSettings};
use screenwide_scenes::rotation::{inverted, turned_homography, Homography};
use screenwide_scenes::SceneRotation;

/// The rows `scene_turn_rows` fills.
pub(super) const SCENE_TURN_ROWS: usize = 9;

const IDENTITY: Homography = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Sends every point behind the eye: an edge-on screen covers no pixel.
const NOWHERE: Homography = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -1.0]];

/// The rows of the inverse turn, then of the turn, then of the inverse turn
/// the shutter opened on, each row of a homography in `xyz`. The first row's
/// `w` is one where the screen is turned at all over the frame, the seventh's
/// where the turn moved while the shutter was open. The inverse turns are
/// scaled so their last entry is one, so the shader can blend them entry by
/// entry.
pub(super) fn scene_turn_rows(
  settings: &ScreenshotOutputSettings,
  placement: &OutputPlacement,
  (width, height): (u32, u32),
) -> [[f32; 4]; SCENE_TURN_ROWS] {
  let extent = f64::from(width.max(height));
  let centre = (
    f64::from(placement.crop_x) + f64::from(placement.crop_width) / 2.0,
    f64::from(placement.crop_y) + f64::from(placement.crop_height) / 2.0,
  );
  let drawn = settings.rotation.unwrap_or(SceneRotation::UPRIGHT);
  let (inverse, turn) = if drawn.is_upright() {
    (IDENTITY, IDENTITY)
  } else {
    let turn = turned_homography(drawn, centre, extent);
    (inverted(turn).map_or(NOWHERE, normalised), turn)
  };
  // Where the screen's box was when the shutter opened: its middle carried
  // by the motion's scale and shift, both in shares of the canvas.
  let opened = settings
    .scene_motion
    .filter(|motion| motion.samples > 1 && motion.rotation != drawn)
    .map(|motion| {
      let [scale, shift_x, shift_y] = motion.screen;
      let centre = (
        centre.0 * scale + shift_x * f64::from(width),
        centre.1 * scale + shift_y * f64::from(height),
      );
      inverted(turned_homography(motion.rotation, centre, extent)).map_or(NOWHERE, normalised)
    });
  let flag = |on: bool| f32::from(u8::from(on));
  let row = |h: Homography, index: usize, last: f32| {
    let [a, b, c] = h[index].map(|value| value as f32);
    [a, b, c, last]
  };
  let opened_inverse = opened.unwrap_or(inverse);
  [
    row(inverse, 0, flag(!drawn.is_upright() || opened.is_some())),
    row(inverse, 1, 0.0),
    row(inverse, 2, 0.0),
    row(turn, 0, 0.0),
    row(turn, 1, 0.0),
    row(turn, 2, 0.0),
    row(opened_inverse, 0, flag(opened.is_some())),
    row(opened_inverse, 1, 0.0),
    row(opened_inverse, 2, 0.0),
  ]
}

/// `h` scaled so its last entry is one in size, where it can be. The scale
/// keeps its sign: the shader reads a point's weight's sign as which side of
/// the eye it lands on.
fn normalised(h: Homography) -> Homography {
  let last = h[2][2].abs();
  if last < 1e-12 {
    return h;
  }
  h.map(|row| row.map(|value| value / last))
}
