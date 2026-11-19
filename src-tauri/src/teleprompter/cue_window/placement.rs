// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Where the cue stands beside the teleprompter, and how large.

use serde::Deserialize;
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use ts_rs::TS;

/// Between the teleprompter and the cue, in points.
const GAP: f64 = 8.0;
/// The least room beside the teleprompter a cue stands in, in points.
const MIN_WIDTH: f64 = 200.0;

/// Where the teleprompter's words are read, in points of its window, for the
/// cue to stand level with them, a glance to the side.
#[derive(Clone, Copy, Debug, Default, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterReadingLine")]
pub struct ReadingLine {
  /// From the window's top to the top of the frame the text runs in.
  pub frame_top: f64,
  /// How far down that frame the line stands, as a share of its height.
  pub share: f64,
  /// From the line to the middle of the words on it.
  pub offset: f64,
  /// Whether the end side is the left, as in a right-to-left language.
  pub end_is_left: bool,
}

/// What the cue's window is placed for.
pub(super) struct Wanted {
  /// The picture's size, in pixels: the window grows no larger, which would
  /// only blur it.
  pub(super) picture: (u32, u32),
  /// The height kept from a resize by hand, in points.
  pub(super) kept: Option<f64>,
  pub(super) line: ReadingLine,
}

/// Where a window stands and how large, in pixels.
pub(super) type Placement = (PhysicalPosition<i32>, PhysicalSize<u32>);

/// Where the cue stands beside `teleprompter`, and how large: on its end
/// side, or its start side where only that has room, as tall as the
/// teleprompter or the height kept and as wide as the picture's shape asks
/// within the room, its middle level with the line being read. None where
/// neither side has room, as when the teleprompter fills its display.
pub(super) fn beside(
  teleprompter: &WebviewWindow,
  wanted: &Wanted,
) -> Result<Option<Placement>, String> {
  let fail = |error: tauri::Error| error.to_string();
  let Some(monitor) = teleprompter.current_monitor().map_err(fail)? else {
    return Ok(None);
  };
  let scale = monitor.scale_factor();
  let area = monitor.work_area();
  let (left, top) = (f64::from(area.position.x), f64::from(area.position.y));
  let (right, bottom) = (
    left + f64::from(area.size.width),
    top + f64::from(area.size.height),
  );
  let position = teleprompter.outer_position().map_err(fail)?;
  let size = teleprompter.outer_size().map_err(fail)?;
  let (x, y) = (f64::from(position.x), f64::from(position.y));
  let (width, height) = (f64::from(size.width), f64::from(size.height));
  let gap = GAP * scale;
  let room_right = right - (x + width) - gap;
  let room_left = x - left - gap;
  let (room_end, room_start) = if wanted.line.end_is_left {
    (room_left, room_right)
  } else {
    (room_right, room_left)
  };
  let at_end = room_end >= MIN_WIDTH * scale;
  let room = if at_end { room_end } else { room_start };
  if room < MIN_WIDTH * scale {
    return Ok(None);
  }
  let (picture_width, picture_height) = (
    f64::from(wanted.picture.0.max(1)),
    f64::from(wanted.picture.1.max(1)),
  );
  let aspect = picture_width / picture_height;
  let mut cue_height = wanted
    .kept
    .map_or(height, |kept| kept * scale)
    .min(bottom - top)
    .min(picture_height);
  let mut cue_width = cue_height * aspect;
  if cue_width > room {
    cue_width = room;
    cue_height = cue_width / aspect;
  }
  let on_right = at_end != wanted.line.end_is_left;
  let cue_x = if on_right {
    x + width + gap
  } else {
    x - gap - cue_width
  };
  let line = &wanted.line;
  let frame_top = line.frame_top * scale;
  let reading = y + frame_top + line.share * (height - frame_top) + line.offset * scale;
  let cue_y = (reading - cue_height / 2.0)
    .min(bottom - cue_height)
    .max(top);
  Ok(Some((
    PhysicalPosition::new(cue_x.round() as i32, cue_y.round() as i32),
    PhysicalSize::new(cue_width.round() as u32, cue_height.round() as u32),
  )))
}
