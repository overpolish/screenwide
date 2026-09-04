// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use tauri::AppHandle;

use crate::screenshots::{CapturedImage, ScreenshotTarget};

mod cancel;
mod canvas;
pub(crate) mod command;
mod fixed_regions;
mod matcher;
mod overlay;
#[cfg(target_os = "macos")]
#[path = "scrolling/platform_macos.rs"]
mod platform;
mod progress;
mod scan;

use scan::{prepare_frame, scan_canvas, seek_boundary};
#[cfg(target_os = "windows")]
#[path = "scrolling/platform_windows.rs"]
mod platform;

// Every document row must be covered by at least three tiles, so that the
// right-edge reconstruction always has two clean samples to outvote a tile
// whose scrollbar thumb happens to sit over that row. A row is covered by
// 1 / SCROLL_FRACTION tiles, so a third of a viewport per step clears the
// majority with room to spare - and leaves ample shared content for matching
// pages with sticky headers. Acquisition remains fast because each pair's
// matching overlaps the next scroll's settle and the settle itself is short.
const SCROLL_FRACTION: f64 = 0.30;
// Boundary frames are only used to find the document origin, so they do not
// need the three-way overlap required by reconstruction. Keep some overlap for
// reliable movement matching while reaching the top-left substantially faster.
const BOUNDARY_SEEK_FRACTION: f64 = 0.70;
const SETTLE_DELAY: Duration = Duration::from_millis(80);
/// Carried by the ordinary error path when Escape stops a capture. The text is
/// never shown - `cancel::was_requested` is what the command believes - but a
/// stop still has to unwind like any other early return.
const CANCELLED: &str = "The scrolling capture was cancelled";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Axis {
  Horizontal,
  Vertical,
}

impl Axis {
  fn length(self, image: &CapturedImage) -> u32 {
    match self {
      Self::Horizontal => image.width,
      Self::Vertical => image.height,
    }
  }

  fn cross_length(self, image: &CapturedImage) -> u32 {
    match self {
      Self::Horizontal => image.height,
      Self::Vertical => image.width,
    }
  }

  /// Maps one overlap sample into the previous and current frames. Keeping
  /// this shared prevents alignment and fixed-region detection from drifting.
  fn mapped_points(
    self,
    direction: Direction,
    shift: u32,
    along: u32,
    across: u32,
  ) -> ((u32, u32), (u32, u32)) {
    match (self, direction) {
      (Self::Vertical, Direction::Forward) => ((across, along + shift), (across, along)),
      (Self::Vertical, Direction::Backward) => ((across, along), (across, along + shift)),
      (Self::Horizontal, Direction::Forward) => ((along + shift, across), (along, across)),
      (Self::Horizontal, Direction::Backward) => ((along, across), (along + shift, across)),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
  Backward,
  Forward,
}

impl Direction {
  fn sign(self) -> i32 {
    match self {
      Self::Backward => -1,
      Self::Forward => 1,
    }
  }

  fn opposite(self) -> Self {
    match self {
      Self::Backward => Self::Forward,
      Self::Forward => Self::Backward,
    }
  }
}

#[derive(Clone, Copy)]
pub(super) struct ScreenPoint {
  x: f64,
  y: f64,
}

fn scroll_geometry(
  app: &AppHandle,
  target: ScreenshotTarget,
) -> Result<(ScreenPoint, f64), String> {
  let ScreenshotTarget::Region { monitor_id, region } = target else {
    return Err("Scrolling capture requires a region".to_owned());
  };
  let (_, scale, monitor) = crate::capture_overlays::monitor_layout(app)?
    .into_iter()
    .find(|(candidate, _, _)| *candidate == monitor_id)
    .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
  #[cfg(target_os = "macos")]
  let origin = monitor.position().to_logical::<f64>(scale);
  #[cfg(target_os = "windows")]
  let origin = monitor.position().cast::<f64>();
  #[cfg(target_os = "macos")]
  let region_scale = 1.0;
  #[cfg(target_os = "windows")]
  let region_scale = scale;
  Ok((
    ScreenPoint {
      x: origin.x + (region.position.x + region.size.width / 2.0) * region_scale,
      y: origin.y + (region.position.y + region.size.height / 2.0) * region_scale,
    },
    scale,
  ))
}

async fn capture_frame(target: ScreenshotTarget) -> Result<CapturedImage, String> {
  super::capture_excluding_own_windows(target).await
}

async fn scroll_once(
  target: ScreenshotTarget,
  point: ScreenPoint,
  axis: Axis,
  direction: Direction,
  logical_amount: u32,
) -> Result<CapturedImage, String> {
  tauri::async_runtime::spawn_blocking(move || {
    platform::send_scroll(point, axis, direction, logical_amount)
  })
  .await
  .map_err(|error| error.to_string())??;
  tokio::time::sleep(SETTLE_DELAY).await;
  capture_frame(target).await
}

async fn capture_canvas(
  app: &AppHandle,
  target: ScreenshotTarget,
) -> Result<CapturedImage, String> {
  let (point, scale) = scroll_geometry(app, target)?;
  let original_pointer = platform::place_pointer(point)?;
  let result = async {
    progress::emit(app, progress::WORKING);
    tokio::time::sleep(Duration::from_millis(120)).await;
    let first = capture_frame(target).await?;
    let horizontal_amount =
      (f64::from(first.width) / scale * BOUNDARY_SEEK_FRACTION).round() as u32;
    let vertical_amount = (f64::from(first.height) / scale * BOUNDARY_SEEK_FRACTION).round() as u32;
    let first = prepare_frame(first).await?;
    let top = seek_boundary(target, point, Axis::Vertical, vertical_amount, scale, first).await?;
    let top_left = seek_boundary(
      target,
      point,
      Axis::Horizontal,
      horizontal_amount,
      scale,
      top,
    )
    .await?;
    let frames = scan_canvas(app, target, point, scale, top_left).await?;
    progress::emit(app, progress::STITCHING);
    let matched = canvas::collect_matches(frames).await?;
    tauri::async_runtime::spawn_blocking(move || canvas::align_and_compose(matched))
      .await
      .map_err(|error| error.to_string())?
  }
  .await;
  platform::restore_pointer(original_pointer);
  result
}
