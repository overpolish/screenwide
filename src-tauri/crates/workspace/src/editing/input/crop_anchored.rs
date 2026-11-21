// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A crop window held about its middle, as the crop of a camera that follows
//! the face is: the face is held in the middle of its box, so the crop only
//! picks how much of the picture shows around it. A pulled side travels and
//! the opposite side goes as far the other way, so the middle never moves.

use super::*;

// The gesture's edge bits.
const FRAME_EDGE_LEFT: u32 = 1;
const FRAME_EDGE_RIGHT: u32 = 2;
const FRAME_EDGE_TOP: u32 = 4;
const FRAME_EDGE_BOTTOM: u32 = 8;

/// `crop` resized about its middle by a pull of `edges` by `delta`, as
/// shares of its pane, inside `image`. A crop that `keeps_shape` grows both
/// ways together, by the larger pull where a corner is pulled. Neither side
/// passes the picture, nor shrinks below `minimum`.
pub(super) fn resize(
  crop: NormalizedRect,
  image: NormalizedRect,
  edges: u32,
  delta: (f64, f64),
  keeps_shape: bool,
  minimum: (f64, f64),
) -> NormalizedRect {
  let half = (crop.width / 2.0, crop.height / 2.0);
  let middle = (crop.x + half.0, crop.y + half.1);
  // How far each half reaches before a side would leave the picture.
  let reach = (
    (middle.0 - image.x)
      .min(image.x + image.width - middle.0)
      .max(0.0),
    (middle.1 - image.y)
      .min(image.y + image.height - middle.1)
      .max(0.0),
  );
  let pull = |low: u32, high: u32, travel: f64| {
    if edges & low != 0 {
      -travel
    } else if edges & high != 0 {
      travel
    } else {
      0.0
    }
  };
  let pulled = (
    half.0 + pull(FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT, delta.0),
    half.1 + pull(FRAME_EDGE_TOP, FRAME_EDGE_BOTTOM, delta.1),
  );
  let across = edges & (FRAME_EDGE_LEFT | FRAME_EDGE_RIGHT) != 0;
  let down = edges & (FRAME_EDGE_TOP | FRAME_EDGE_BOTTOM) != 0;
  let next = if keeps_shape {
    let scale_x = pulled.0 / half.0.max(f64::EPSILON);
    let scale_y = pulled.1 / half.1.max(f64::EPSILON);
    let scale = match (across, down) {
      (true, true) => scale_x.max(scale_y),
      (true, false) => scale_x,
      (false, true) => scale_y,
      (false, false) => 1.0,
    };
    let smallest =
      (minimum.0 / crop.width.max(f64::EPSILON)).max(minimum.1 / crop.height.max(f64::EPSILON));
    let largest = (reach.0 / half.0.max(f64::EPSILON)).min(reach.1 / half.1.max(f64::EPSILON));
    let scale = scale.max(smallest).min(largest);
    (half.0 * scale, half.1 * scale)
  } else {
    (
      pulled.0.max(minimum.0 / 2.0).min(reach.0),
      pulled.1.max(minimum.1 / 2.0).min(reach.1),
    )
  };
  NormalizedRect {
    x: middle.0 - next.0,
    y: middle.1 - next.1,
    width: next.0 * 2.0,
    height: next.1 * 2.0,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const IMAGE: NormalizedRect = NormalizedRect {
    x: 0.0,
    y: 0.0,
    width: 1.0,
    height: 1.0,
  };
  const CROP: NormalizedRect = NormalizedRect {
    x: 0.4,
    y: 0.3,
    width: 0.2,
    height: 0.4,
  };

  #[test]
  fn a_pulled_side_moves_the_opposite_one_back() {
    let next = resize(
      CROP,
      IMAGE,
      FRAME_EDGE_RIGHT,
      (0.05, 0.0),
      false,
      (0.01, 0.01),
    );
    assert!((next.x - 0.35).abs() < 1e-9 && (next.width - 0.3).abs() < 1e-9);
    assert!((next.y - CROP.y).abs() < 1e-9 && (next.height - CROP.height).abs() < 1e-9);
  }

  #[test]
  fn a_shaped_crop_grows_both_ways_and_stops_at_the_picture() {
    let next = resize(
      CROP,
      IMAGE,
      FRAME_EDGE_RIGHT,
      (0.5, 0.0),
      true,
      (0.01, 0.01),
    );
    // The height reaches the picture first: 0.3 above the middle's 0.5.
    assert!((next.height - 1.0).abs() < 1e-9 && (next.width - 0.5).abs() < 1e-9);
    assert!((next.x + next.width / 2.0 - 0.5).abs() < 1e-9);
  }
}
