// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the presets that pair the screen and the camera lay them out on a
//! canvas taller than it is wide, as a short video is. The screen comes
//! first: it is shown whole at its own shape, as wide as the margins allow,
//! and inside the part of the canvas the platform's interface leaves clear.
//! The camera is the pane that gives way. The twins of `tallPair` and
//! `tallBubble` in `src/features/editor/recording/scenes/recording-scene-tall.ts`.

use super::clear_area::ClearArea;
use super::geometry::Rect;
use super::variant::{SceneBubbleSize, SceneCameraSize, SceneCorner, SceneVariant};

/// The camera's length in a pair over the screen's, at the least.
fn camera_ratio(size: SceneCameraSize) -> f64 {
  match size {
    SceneCameraSize::Third => 0.5,
    SceneCameraSize::TwoThirds => 2.0,
  }
}

/// A picture in picture's camera's side, as a share of the screen's width.
fn bubble_share(size: SceneBubbleSize) -> f64 {
  match size {
    SceneBubbleSize::Small => 0.34,
    SceneBubbleSize::Large => 0.5,
  }
}

/// How much of the camera hangs past the screen's edge, as a share of its
/// side.
const BUBBLE_HANG: f64 = 0.75;

/// The margin kept at the canvas's sides, at least the clear area's.
fn side_margin(width: f64, clear: ClearArea) -> f64 {
  (width / 18.0).max(clear.side * width)
}

/// Side by side or stacked: the screen whole and as wide as the margins
/// allow, the camera directly under it (over it where swapped) at the same
/// width, taking the rest of the clear area and at least its share of the
/// pair. A camera two thirds of the pair keeps them by making the screen
/// smaller, and may run on under the platform's bar on its side.
pub(crate) fn pair(
  canvas: (f64, f64),
  screen_aspect: f64,
  variant: SceneVariant,
  clear: ClearArea,
) -> (Rect, Rect) {
  let (canvas_width, canvas_height) = canvas;
  let margin = canvas_width / 18.0;
  let gap = margin / 2.0;
  let ratio = camera_ratio(variant.camera_size.unwrap_or_default());
  let (clear_top, clear_bottom) = clear.rows(canvas_height, margin);
  let clear_height = clear_bottom - clear_top;
  let reach = if variant.swap {
    clear_bottom - margin
  } else {
    canvas_height - margin - clear_top
  };
  let height = ((canvas_width - side_margin(canvas_width, clear) * 2.0) / screen_aspect)
    .min(clear_height)
    .min((reach - gap) / (1.0 + ratio));
  let width = height * screen_aspect;
  let camera_height = (height * ratio).max(clear_height - height - gap);
  let total = height + gap + camera_height;
  let top = if total <= clear_height {
    clear_top + (clear_height - total) / 2.0
  } else if variant.swap {
    clear_bottom - total
  } else {
    clear_top
  };
  let (screen_y, camera_y) = if variant.swap {
    (top + camera_height + gap, top)
  } else {
    (top, top + height + gap)
  };
  let x = (canvas_width - width) / 2.0;
  (
    Rect {
      x,
      y: screen_y,
      width,
      height,
    },
    Rect {
      x,
      y: camera_y,
      width,
      height: camera_height,
    },
  )
}

/// Picture in picture: the screen whole and as wide as the margins allow,
/// the round camera hanging below it for a bottom corner and above it for a
/// top one, on its corner's side, just overlapping the screen's edge. The
/// pair sits in the middle of the clear area.
pub(crate) fn bubble(
  canvas: (f64, f64),
  screen_aspect: f64,
  variant: SceneVariant,
  clear: ClearArea,
) -> (Rect, Rect) {
  let (canvas_width, canvas_height) = canvas;
  let (clear_top, clear_bottom) = clear.rows(canvas_height, canvas_width / 18.0);
  let clear_height = clear_bottom - clear_top;
  let share = bubble_share(variant.size.unwrap_or_default());
  let width = (canvas_width - side_margin(canvas_width, clear) * 2.0)
    .min(clear_height / (1.0 / screen_aspect + share * BUBBLE_HANG));
  let height = width / screen_aspect;
  let side = width * share;
  let top = clear_top + (clear_height - height - side * BUBBLE_HANG) / 2.0;
  let x = (canvas_width - width) / 2.0;
  let corner = variant.corner.unwrap_or_default();
  let below = matches!(corner, SceneCorner::BottomLeft | SceneCorner::BottomRight);
  let right = matches!(corner, SceneCorner::TopRight | SceneCorner::BottomRight);
  let screen_y = if below { top } else { top + side * BUBBLE_HANG };
  (
    Rect {
      x,
      y: screen_y,
      width,
      height,
    },
    Rect {
      x: if right {
        x + width - side * 1.1
      } else {
        x + side * 0.1
      },
      y: if below {
        screen_y + height - side * (1.0 - BUBBLE_HANG)
      } else {
        top
      },
      width: side,
      height: side,
    },
  )
}
