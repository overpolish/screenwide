// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use tauri::AppHandle;

use super::{
  cancel,
  canvas::{self, CapturedStep, Movement},
  matcher::{self, MatchFrame},
  progress, scroll_once, Axis, Direction, ScreenPoint, CANCELLED, SCROLL_FRACTION,
};
use crate::screenshots::{CapturedImage, ScreenshotTarget};

/// How many backward scroll attempts may be made on each axis before giving
/// up. Boundary seeking uses larger steps than capture, so this is deliberately
/// independent of the retained-tile ceiling below.
const MAX_BOUNDARY_STEPS: usize = 128;
/// Retained frames dominate the memory a capture holds, and 96 of them already
/// approaches a gigabyte, so the tile ceiling stays put even as smaller steps
/// shorten the page it can reach.
const MAX_TILES: usize = 96;

/// Derives the matching planes for a freshly captured frame.
///
/// Plane construction touches every pixel, so it belongs on a blocking thread
/// rather than on the async runtime that is also driving the scroll.
pub(super) async fn prepare_frame(image: CapturedImage) -> Result<MatchFrame, String> {
  tauri::async_runtime::spawn_blocking(move || MatchFrame::new(Arc::new(image)))
    .await
    .map_err(|error| error.to_string())
}

async fn advance(current: MatchFrame, image: CapturedImage) -> Result<(MatchFrame, bool), String> {
  tauri::async_runtime::spawn_blocking(move || {
    let next = MatchFrame::new(Arc::new(image));
    let unchanged = matcher::frames_are_same(&current, &next);
    (next, unchanged)
  })
  .await
  .map_err(|error| error.to_string())
}

fn spawn_pair(previous: MatchFrame, current: MatchFrame, movement: Movement) -> CapturedStep {
  let image = current.image.clone();
  let pending =
    tauri::async_runtime::spawn_blocking(move || canvas::match_pair(previous, current, movement));
  CapturedStep {
    image,
    movement: Some(movement),
    pending: Some(pending),
  }
}

pub(super) async fn seek_boundary(
  target: ScreenshotTarget,
  point: ScreenPoint,
  axis: Axis,
  logical_amount: u32,
  scale: f64,
  mut current: MatchFrame,
) -> Result<MatchFrame, String> {
  let expected = (f64::from(logical_amount) * scale).round() as u32;
  let mut uncertain = false;
  for _ in 0..MAX_BOUNDARY_STEPS {
    if cancel::is_requested() {
      return Err(CANCELLED.to_owned());
    }
    let image = scroll_once(target, point, axis, Direction::Backward, logical_amount).await?;
    let (next, unchanged) = advance(current.clone(), image).await?;
    // Boundary seeking is inherently serial, but the matcher still costs tens
    // of milliseconds of CPU that must not stall the runtime.
    let matched = {
      let previous = current.clone();
      let candidate = next.clone();
      tauri::async_runtime::spawn_blocking(move || {
        matcher::find_shift(&previous, &candidate, axis, Direction::Backward, expected)
      })
      .await
      .map_err(|error| error.to_string())?
    };
    if matched.is_some() {
      current = next;
      uncertain = false;
      continue;
    }
    if unchanged || uncertain {
      return Ok(next);
    }
    uncertain = true;
    current = next;
  }
  Err("The scrollable region is too large to find its beginning".to_owned())
}

pub(super) async fn scan_canvas(
  app: &AppHandle,
  target: ScreenshotTarget,
  point: ScreenPoint,
  scale: f64,
  first: MatchFrame,
) -> Result<Vec<CapturedStep>, String> {
  let horizontal_amount = (f64::from(first.image.width) / scale * SCROLL_FRACTION).round() as u32;
  let vertical_amount = (f64::from(first.image.height) / scale * SCROLL_FRACTION).round() as u32;
  let horizontal_expected = (f64::from(horizontal_amount) * scale).round() as u32;
  let vertical_expected = (f64::from(vertical_amount) * scale).round() as u32;
  let mut frames = vec![CapturedStep {
    image: first.image.clone(),
    movement: None,
    pending: None,
  }];
  let mut current = first;
  let mut horizontal_direction = Direction::Forward;
  let mut horizontal_scrollable = true;
  progress::emit(app, progress::CAPTURING);

  loop {
    if cancel::is_requested() {
      return Err(CANCELLED.to_owned());
    }
    if horizontal_scrollable {
      let mut moved_in_row = false;
      loop {
        if cancel::is_requested() {
          return Err(CANCELLED.to_owned());
        }
        let image = scroll_once(
          target,
          point,
          Axis::Horizontal,
          horizontal_direction,
          horizontal_amount,
        )
        .await?;
        let (next, unchanged) = advance(current.clone(), image).await?;
        if unchanged {
          break;
        }
        frames.push(spawn_pair(
          current,
          next.clone(),
          Movement {
            axis: Axis::Horizontal,
            direction: horizontal_direction,
            expected: horizontal_expected,
          },
        ));
        current = next;
        moved_in_row = true;
        if frames.len() >= MAX_TILES {
          return Err("The scrolling capture is too large".to_owned());
        }
      }
      if frames.len() == 1 && !moved_in_row {
        horizontal_scrollable = false;
      }
    }

    let image = scroll_once(
      target,
      point,
      Axis::Vertical,
      Direction::Forward,
      vertical_amount,
    )
    .await?;
    let (next, unchanged) = advance(current.clone(), image).await?;
    if unchanged {
      break;
    }
    frames.push(spawn_pair(
      current,
      next.clone(),
      Movement {
        axis: Axis::Vertical,
        direction: Direction::Forward,
        expected: vertical_expected,
      },
    ));
    current = next;
    if frames.len() >= MAX_TILES {
      return Err("The scrolling capture is too large".to_owned());
    }
    horizontal_direction = horizontal_direction.opposite();
  }

  Ok(frames)
}
