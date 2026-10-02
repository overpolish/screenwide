// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How far the panes move while a frame's shutter is open. A scene's panes
//! glide between placements, and a frame that only drew where they ended up
//! would step them across the canvas. The compositor instead draws the screen
//! and the camera at every step between where they were when the shutter
//! opened and where the frame draws them, and averages the steps, the way a
//! moving annotation is blurred. The background stays sharp.

use super::geometry::Rect;
use super::placement::Placement;

/// The farthest apart two consecutive steps may land, in output pixels, and
/// the most steps one frame takes. The screen is sampled whole at every step,
/// so the cap is kept below the annotations' own.
const STEP_PIXELS: f64 = 0.75;
const MAX_SAMPLES: f64 = 24.0;

/// Where the panes were when a frame's shutter opened, relative to where the
/// frame draws them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneMotion {
  /// The scale and the shift that carry the screen's box as drawn onto where
  /// it was. The box keeps its shape, so it only moves and scales. The shift
  /// is in shares of the canvas, so a canvas drawn at another size keeps it.
  pub screen: [f64; 3],
  /// The same for the screen's image, its crop and the annotations on it. A
  /// zoom scales the image further than its box, so the two move apart.
  pub image: [f64; 3],
  /// The camera's frame then, in shares of the canvas.
  pub camera_frame: [f64; 4],
  /// The part of the camera picture that frame showed, in shares of the
  /// picture. A scene changes the frame's shape, so the camera does not move
  /// as one piece the way the screen does.
  pub camera_crop: [f64; 4],
  /// How many steps the frame averages, the frame's own placement included.
  pub samples: u32,
}

fn travel(a: Rect, b: Rect) -> f64 {
  (a.x - b.x)
    .abs()
    .max((a.y - b.y).abs())
    .max((a.x + a.width - b.x - b.width).abs())
    .max((a.y + a.height - b.y - b.height).abs())
}

impl SceneMotion {
  /// The motion from `opened` to `drawn` on a `canvas` for a camera picture of
  /// `camera_aspect`; `None` where the panes moved too little to show it.
  pub(super) fn between(
    opened: &Placement,
    drawn: &Placement,
    canvas: (f64, f64),
    camera_aspect: f64,
  ) -> Option<Self> {
    let image_travel = (opened.image.0 - drawn.image.0)
      .abs()
      .max((opened.image.1 - drawn.image.1).abs())
      + (opened.image.2 - drawn.image.2).abs();
    let moved = travel(opened.screen, drawn.screen)
      .max(travel(opened.frame, drawn.frame))
      .max(image_travel);
    if moved < STEP_PIXELS
      || drawn.screen.width <= 0.0
      || drawn.image.2 <= 0.0
      || canvas.0 <= 0.0
      || canvas.1 <= 0.0
    {
      return None;
    }
    let scale = opened.screen.width / drawn.screen.width;
    let image_scale = opened.image.2 / drawn.image.2;
    let (centre_x, centre_y, width) = opened.camera;
    let width = width.max(f64::EPSILON);
    let height = width / camera_aspect;
    Some(Self {
      screen: [
        scale,
        (opened.screen.x - drawn.screen.x * scale) / canvas.0,
        (opened.screen.y - drawn.screen.y * scale) / canvas.1,
      ],
      image: [
        image_scale,
        (opened.image.0 - drawn.image.0 * image_scale) / canvas.0,
        (opened.image.1 - drawn.image.1 * image_scale) / canvas.1,
      ],
      camera_frame: [
        opened.frame.x / canvas.0,
        opened.frame.y / canvas.1,
        opened.frame.width / canvas.0,
        opened.frame.height / canvas.1,
      ],
      camera_crop: [
        (opened.frame.x - (centre_x - width / 2.0)) / width,
        (opened.frame.y - (centre_y - height / 2.0)) / height,
        opened.frame.width / width,
        opened.frame.height / height,
      ],
      samples: ((moved / STEP_PIXELS).ceil() + 1.0).clamp(2.0, MAX_SAMPLES) as u32,
    })
  }
}

#[cfg(test)]
mod tests;
