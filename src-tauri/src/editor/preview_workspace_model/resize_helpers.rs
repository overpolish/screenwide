// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_compositor::workspace_editor::{
  FRAME_EDGE_BOTTOM, FRAME_EDGE_CENTERED, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT, FRAME_EDGE_TOP,
};
use screenwide_output::{narrowest_in_reach, CanvasAnchor};

/// How far a frame's top left corner moved, in output pixels.
pub(super) fn origin_shift(old: WorldRect, new: WorldRect) -> (f64, f64) {
  (new.x - old.x, new.y - old.y)
}

/// Renumbers a layer's placement so it keeps its place on screen after the
/// canvas's corner moved by `shift`.
pub(super) fn shift_output(output: &mut ScreenshotOutputSettings, shift: (f64, f64)) {
  output.crop_x -= shift.0;
  output.crop_y -= shift.1;
  output.image_x -= shift.0;
  output.image_y -= shift.1;
}

/// How a resize by `edges` moves the canvas's corner (`CanvasAnchor`).
fn resize_anchor(edges: u32) -> CanvasAnchor {
  let share = |near: u32, far: u32| {
    if edges & (near | far) == 0 {
      0.0
    } else if edges & FRAME_EDGE_CENTERED != 0 {
      0.5
    } else if edges & near != 0 {
      1.0
    } else {
      0.0
    }
  };
  CanvasAnchor {
    x: share(FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT),
    y: share(FRAME_EDGE_TOP, FRAME_EDGE_BOTTOM),
  }
}

/// `delta`, in output pixels, held to what keeps every one of `placed`
/// within reach of the canvas (`narrowest_in_reach`): a resize never moves
/// or scales what the canvas holds, so a canvas narrowed far enough leaves a
/// picture many times wider than itself, or far off it, which a placement
/// may not be. Each axis stops at its own floor, and the native surface
/// stops its drawing there too, so a drag past it and back again picks up
/// where it stopped. A canvas already smaller than its floor is let resize
/// freely, so it can be grown back.
pub(super) fn held_delta<'a>(
  delta: (f64, f64),
  edges: u32,
  placed: impl IntoIterator<Item = &'a ScreenshotOutputSettings>,
) -> (f64, f64) {
  let anchor = resize_anchor(edges);
  let mut size = (0.0, 0.0);
  let mut floor = (0.0_f64, 0.0_f64);
  for settings in placed {
    size = (f64::from(settings.width), f64::from(settings.height));
    let (width, height) = narrowest_in_reach(settings, anchor);
    floor = (floor.0.max(width), floor.1.max(height));
  }
  let centered = edges & FRAME_EDGE_CENTERED != 0;
  let hold = |delta: f64, size: f64, floor: f64, near: u32, far: u32| {
    if size < floor {
      return delta;
    }
    // How far the moved edge may come in before the canvas is `floor`.
    let room = if centered {
      (size - floor) / 2.0
    } else {
      size - floor
    };
    if edges & near != 0 {
      delta.min(room)
    } else if edges & far != 0 {
      delta.max(-room)
    } else {
      delta
    }
  };
  (
    hold(delta.0, size.0, floor.0, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT),
    hold(delta.1, size.1, floor.1, FRAME_EDGE_TOP, FRAME_EDGE_BOTTOM),
  )
}
