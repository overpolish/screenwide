// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(any(target_os = "windows", test))]
use super::{
  NormalizedRect, FRAME_EDGE_BOTTOM, FRAME_EDGE_CENTERED, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT,
  FRAME_EDGE_TOP,
};

/// `f64::clamp` panics when `low > high`, and two bounds derived from the same
/// normalized geometry (`image.x + image.width - crop.width` against `image.x`
/// while the crop still spans the whole image) can differ by an ULP. An
/// inverted range collapses to `low`.
#[cfg(any(target_os = "windows", test))]
fn clamp_range(value: f64, low: f64, high: f64) -> f64 {
  value.max(low).min(high.max(low))
}

/// Move a crop window without changing the underlying image transform.
#[cfg(any(target_os = "windows", test))]
pub fn apply_crop_move(
  crop: NormalizedRect,
  image: NormalizedRect,
  delta: (f64, f64),
) -> NormalizedRect {
  let x = clamp_range(
    crop.x + delta.0,
    image.x,
    image.x + image.width - crop.width,
  );
  let y = clamp_range(
    crop.y + delta.1,
    image.y,
    image.y + image.height - crop.height,
  );
  NormalizedRect { x, y, ..crop }
}

/// Resize a crop window from edge bits (left=1, right=2, top=4, bottom=8).
/// The image bounds constrain the crop but are never modified.
#[cfg(any(target_os = "windows", test))]
pub fn apply_crop_resize(
  crop: NormalizedRect,
  image: NormalizedRect,
  edges: u32,
  delta: (f64, f64),
  centered: bool,
) -> NormalizedRect {
  let image_right = image.x + image.width;
  let image_bottom = image.y + image.height;
  let min_size = 1e-6;
  let mut left = crop.x;
  let mut right = crop.x + crop.width;
  let mut top = crop.y;
  let mut bottom = crop.y + crop.height;
  if edges & FRAME_EDGE_LEFT != 0 {
    let movement = clamp_range(delta.0, image.x - left, crop.width - min_size);
    left += movement;
    if centered {
      right -= movement;
    }
  } else if edges & FRAME_EDGE_RIGHT != 0 {
    let movement = clamp_range(delta.0, min_size - crop.width, image_right - right);
    right += movement;
    if centered {
      left -= movement;
    }
  }
  if edges & FRAME_EDGE_TOP != 0 {
    let movement = clamp_range(delta.1, image.y - top, crop.height - min_size);
    top += movement;
    if centered {
      bottom -= movement;
    }
  } else if edges & FRAME_EDGE_BOTTOM != 0 {
    let movement = clamp_range(delta.1, min_size - crop.height, image_bottom - bottom);
    bottom += movement;
    if centered {
      top -= movement;
    }
  }
  let width = (right - left).max(min_size).min(image.width.max(min_size));
  let height = (bottom - top).max(min_size).min(image.height.max(min_size));
  left = clamp_range(left, image.x, image_right - width);
  top = clamp_range(top, image.y, image_bottom - height);
  NormalizedRect {
    x: left,
    y: top,
    width,
    height,
  }
}

/// Resize a crop window that keeps its shape: the part of a picture a scene's
/// box shows, which has the box's shape at any zoom. A corner is pulled from
/// the one opposite it and an edge from the middle of the far one, by however
/// far the pointer took it along the axis that grew most. The window stays
/// inside `image` and no smaller than `minimum` on either side.
#[cfg(any(target_os = "windows", test))]
pub fn apply_framed_crop_resize(
  crop: NormalizedRect,
  image: NormalizedRect,
  edges: u32,
  delta: (f64, f64),
  minimum: (f64, f64),
) -> NormalizedRect {
  let width = crop.width.max(f64::EPSILON);
  let height = crop.height.max(f64::EPSILON);
  // Where the anchor sits in the window, along each axis, as a share of it.
  let share = |low: u32, high: u32| {
    if edges & low != 0 {
      1.0
    } else if edges & high != 0 {
      0.0
    } else {
      0.5
    }
  };
  let share_x = share(FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT);
  let share_y = share(FRAME_EDGE_TOP, FRAME_EDGE_BOTTOM);
  let grown = |share: f64, delta: f64, length: f64| {
    (length + if share == 1.0 { -delta } else { delta }) / length
  };
  let scale = match (share_x != 0.5, share_y != 0.5) {
    (true, true) => grown(share_x, delta.0, width).max(grown(share_y, delta.1, height)),
    (true, false) => grown(share_x, delta.0, width),
    _ => grown(share_y, delta.1, height),
  };
  let anchor_x = crop.x + share_x * width;
  let anchor_y = crop.y + share_y * height;
  // The longest the window may run each way from its anchor inside the image.
  let room = |anchor: f64, share: f64, low: f64, high: f64| {
    if share == 1.0 {
      anchor - low
    } else if share == 0.0 {
      high - anchor
    } else {
      2.0 * (anchor - low).min(high - anchor)
    }
  };
  let largest = (room(anchor_x, share_x, image.x, image.x + image.width) / width)
    .min(room(anchor_y, share_y, image.y, image.y + image.height) / height);
  let smallest = (minimum.0 / width).max(minimum.1 / height).min(largest);
  let scale = scale.max(smallest).min(largest);
  NormalizedRect {
    x: anchor_x - share_x * width * scale,
    y: anchor_y - share_y * height * scale,
    width: width * scale,
    height: height * scale,
  }
}

/// One sample of a crop window drawn from a press outside the current one.
/// `delta` is what the gesture reports: the far corner's offset from the
/// anchor, or the half extents about it when `edges` carries the centered
/// bit. The anchor is always a corner or the centre of `rect`, which is what
/// lets the frontend rebuild the same rectangle from the anchor and `delta`.
#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropDraw {
  pub rect: NormalizedRect,
  pub delta: (f64, f64),
  pub edges: u32,
}

/// Draw a crop window from `anchor` towards `pointer` inside `image`. Both
/// sides are at least `minimum` long where the image leaves room, so a short
/// drag or one along a single axis never collapses the crop. Centering is
/// honoured only while the anchor has half a minimum of room on every side.
#[cfg(any(target_os = "windows", test))]
pub fn apply_crop_draw(
  anchor: (f64, f64),
  pointer: (f64, f64),
  image: NormalizedRect,
  minimum: (f64, f64),
  centered: bool,
) -> CropDraw {
  let axes = [
    (
      anchor.0,
      pointer.0,
      image.x,
      image.x + image.width,
      minimum.0,
    ),
    (
      anchor.1,
      pointer.1,
      image.y,
      image.y + image.height,
      minimum.1,
    ),
  ];
  let centered = centered
    && axes
      .iter()
      .all(|&(anchor, _, low, high, minimum)| (anchor - low).min(high - anchor) >= minimum / 2.0);
  let [x, y] = axes.map(|(anchor, pointer, low, high, minimum)| {
    let offset = clamp_range(pointer, low, high) - anchor;
    let toward = if offset < 0.0 { -1.0 } else { 1.0 };
    if centered {
      let room = (anchor - low).min(high - anchor);
      return toward * offset.abs().max(minimum / 2.0).min(room);
    }
    let length = offset.abs().max(minimum);
    let reach = |direction: f64| clamp_range(anchor + direction * length, low, high) - anchor;
    let forward = reach(toward);
    let backward = reach(-toward);
    // Pressing against an edge and dragging into it would leave a sliver;
    // the window opens away from that edge instead.
    if forward.abs() < minimum && backward.abs() > forward.abs() {
      backward
    } else {
      forward
    }
  });
  let span = |anchor: f64, delta: f64| {
    if centered {
      (anchor - delta.abs(), 2.0 * delta.abs())
    } else {
      (anchor.min(anchor + delta), delta.abs())
    }
  };
  let (left, width) = span(anchor.0, x);
  let (top, height) = span(anchor.1, y);
  let horizontal = if x < 0.0 {
    FRAME_EDGE_LEFT
  } else {
    FRAME_EDGE_RIGHT
  };
  let vertical = if y < 0.0 {
    FRAME_EDGE_TOP
  } else {
    FRAME_EDGE_BOTTOM
  };
  CropDraw {
    rect: NormalizedRect {
      x: left,
      y: top,
      width,
      height,
    },
    delta: (x, y),
    edges: horizontal | vertical | if centered { FRAME_EDGE_CENTERED } else { 0 },
  }
}

#[cfg(test)]
mod framed_tests;
