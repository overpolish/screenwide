// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A turned layer resized by one of its grips, worked out where it is drawn.
//!
//! A layer is turned about its own middle and seen from the canvas's
//! distance, so scaling its upright box about a corner moves its middle and
//! with it the whole turned picture: the held corner would slide on screen
//! and the dragged one run away from the hand. Instead the held point stays
//! where the press found it drawn, and the scale is the one that brings the
//! dragged grip, as drawn, nearest the hand. Upright, that is the projection
//! onto the grip's diagonal an upright resize takes.

use super::*;

/// What a resize of a turned layer is measured against, as the press found
/// it: the turn, the layer's upright box on screen and the canvas it is laid
/// out on, whose longer side the turn is seen from, both in display points.
#[derive(Clone, Copy)]
pub struct TurnedStart {
  rotation: SceneRotation,
  frame: PreviewSurfaceRect,
  pane: PreviewSurfaceRect,
}

/// `selection` as a resize measures it: `None` where it stands upright, is
/// seen edge on, or has no pane laid out.
pub(in super::super) fn turned_start(
  state: &SurfaceState,
  selection: PreviewSelection,
) -> Option<TurnedStart> {
  homographies(state, selection)?;
  Some(TurnedStart {
    rotation: rotation(selection),
    frame: display_selection(state, selection)?,
    pane: state
      .workspace_transform
      .apply(state.viewport, selection_pane_rect(state, selection)),
  })
}

/// How finely the scales between the smallest and the largest are first
/// tried, before the nearest is narrowed down.
const SCAN_STEPS: i32 = 64;

/// How many times the nearest scale's bracket is narrowed, each by the
/// golden ratio.
const NARROWING_STEPS: usize = 48;

/// The scale a grip moving `edges` reaches when the hand is at `point`, and
/// the box's new top-left corner as shares of the pane, as an upright resize
/// reports them. The held point is the opposite corner or side, or the
/// middle where `centered`. `limits` are the smallest and largest scale.
/// `None` where no scale keeps the layer in front of the eye.
pub(in super::super) fn turned_resize(
  start: TurnedStart,
  edges: u32,
  centered: bool,
  point: (f64, f64),
  (minimum, maximum): (f64, f64),
) -> Option<(f64, (f64, f64))> {
  let TurnedStart {
    rotation,
    frame,
    pane,
  } = start;
  // The turn about the origin: `middle + turn(offset)` is where a point
  // `offset` from the layer's middle is drawn.
  let turn = turned_homography(rotation, (0.0, 0.0), pane.width.max(pane.height));
  let drawn = |(x, y): (f64, f64), scale: f64| apply(turn, (x * scale, y * scale));
  let half = (frame.width / 2.0, frame.height / 2.0);
  let side = |low: u32, high: u32| {
    if edges & low != 0 {
      -1.0
    } else if edges & high != 0 {
      1.0
    } else {
      0.0
    }
  };
  let grip = (side(1, 2) * half.0, side(4, 8) * half.1);
  let anchor = if centered {
    (0.0, 0.0)
  } else {
    (-grip.0, -grip.1)
  };
  let reach = drawn(anchor, 1.0)?;
  let held = (frame.x + half.0 + reach.0, frame.y + half.1 + reach.1);
  // At `scale`, the middle that keeps the held point where it is drawn, and
  // how far the dragged grip is drawn from the hand.
  let at = |scale: f64| -> Option<((f64, f64), f64)> {
    for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
      drawn((x * half.0, y * half.1), scale)?;
    }
    let reach = drawn(anchor, scale)?;
    let middle = (held.0 - reach.0, held.1 - reach.1);
    let to_grip = drawn(grip, scale)?;
    let miss = (
      middle.0 + to_grip.0 - point.0,
      middle.1 + to_grip.1 - point.1,
    );
    Some((middle, miss.0 * miss.0 + miss.1 * miss.1))
  };
  let low = minimum.max(1e-6);
  let high = maximum.max(low);
  let ratio = (high / low).powf(1.0 / f64::from(SCAN_STEPS));
  let scale_at = |step: i32| low * ratio.powi(step.clamp(0, SCAN_STEPS));
  let (nearest, _) = (0..=SCAN_STEPS)
    .filter_map(|step| at(scale_at(step)).map(|(_, miss)| (step, miss)))
    .min_by(|a, b| a.1.total_cmp(&b.1))?;
  let (mut from, mut to) = (scale_at(nearest - 1), scale_at(nearest + 1));
  for _ in 0..NARROWING_STEPS {
    let first = from + (to - from) * 0.381_966;
    let second = from + (to - from) * 0.618_034;
    match (at(first), at(second)) {
      (Some((_, a)), Some((_, b))) if a <= b => to = second,
      (Some(_), None) => to = second,
      (_, Some(_)) => from = first,
      (None, None) => break,
    }
  }
  let mut scale = (from + to) / 2.0;
  let middle = match at(scale) {
    Some((middle, _)) => middle,
    None => {
      scale = scale_at(nearest);
      at(scale)?.0
    }
  };
  Some((
    scale,
    (
      (middle.0 - half.0 * scale - pane.x) / pane.width,
      (middle.1 - half.1 * scale - pane.y) / pane.height,
    ),
  ))
}

#[cfg(test)]
mod tests;
