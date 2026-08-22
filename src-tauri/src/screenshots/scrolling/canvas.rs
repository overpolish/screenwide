// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use tauri::async_runtime::JoinHandle;

use super::{fixed_regions, matcher, Axis, Direction};
use crate::screenshots::CapturedImage;

mod compose;
mod reconstruct;

use compose::{compose, Crop, Tile};

const TRAILING_EDGE_CROP: u32 = 64;
/// Rows (or columns) a pair must still share once both of its crops are taken,
/// so that an over-generous static strip can never open a gap in the canvas.
const MINIMUM_PAIR_OVERLAP: u32 = 8;

#[derive(Clone, Copy)]
pub(super) struct Movement {
  pub(super) axis: Axis,
  pub(super) direction: Direction,
  pub(super) expected: u32,
}

/// The result of analysing one consecutive frame pair.
pub(super) struct PairOutcome {
  regions: fixed_regions::FixedRegions,
  matched: matcher::ShiftMatch,
}

/// A captured frame plus the still-running analysis of the pair it closes.
///
/// Matching a pair costs tens of milliseconds of CPU while the next scroll
/// step spends at least as long settling, so the work is started the moment
/// the frame lands and collected only once the scan has finished.
pub(super) struct CapturedStep {
  pub(super) image: Arc<CapturedImage>,
  pub(super) movement: Option<Movement>,
  pub(super) pending: Option<JoinHandle<PairOutcome>>,
}

pub(super) struct MatchedStep {
  image: Arc<CapturedImage>,
  movement: Option<Movement>,
  outcome: Option<PairOutcome>,
}

pub(super) fn match_pair(
  previous: matcher::MatchFrame,
  current: matcher::MatchFrame,
  movement: Movement,
) -> PairOutcome {
  let matched = matcher::match_shift(
    &previous,
    &current,
    movement.axis,
    movement.direction,
    movement.expected,
  );
  // A rejected pair still becomes a tile, placed at the scroll we asked for, so
  // that one unalignable seam costs a blemish rather than the whole capture.
  let regions = fixed_regions::detect(
    &previous.image,
    &current.image,
    movement.axis,
    movement.direction,
    matched.shift.unwrap_or(movement.expected),
  );
  PairOutcome { regions, matched }
}

/// Collects the per-pair analyses in capture order.
pub(super) async fn collect_matches(steps: Vec<CapturedStep>) -> Result<Vec<MatchedStep>, String> {
  let mut matched = Vec::with_capacity(steps.len());
  for step in steps {
    let outcome = match step.pending {
      Some(pending) => Some(pending.await.map_err(|error| error.to_string())?),
      None => None,
    };
    matched.push(MatchedStep {
      image: step.image,
      movement: step.movement,
      outcome,
    });
  }
  Ok(matched)
}

pub(super) fn align_and_compose(frames: Vec<MatchedStep>) -> Result<CapturedImage, String> {
  let mut frames = frames.into_iter();
  let first = frames
    .next()
    .ok_or_else(|| "The scrolling capture produced no frames".to_owned())?;
  let mut tiles = vec![Tile {
    crop: Crop::default(),
    fixed_bands: None,
    image: first.image,
    x: 0,
    y: 0,
  }];

  for frame in frames {
    let movement = frame
      .movement
      .ok_or_else(|| "A scrolling capture frame has no movement".to_owned())?;
    let outcome = frame
      .outcome
      .ok_or_else(|| "A scrolling capture frame was never matched".to_owned())?;
    let previous = tiles
      .last()
      .ok_or_else(|| "The scrolling capture produced no frames".to_owned())?;
    let PairOutcome { regions, matched } = outcome;
    let shift = matched.shift.unwrap_or(movement.expected);
    let (mut x, mut y) = (previous.x, previous.y);
    match movement.axis {
      Axis::Horizontal => x += movement.direction.sign() * shift as i32,
      Axis::Vertical => y += movement.direction.sign() * shift as i32,
    }
    // A sticky header or toolbar sits at a fixed viewport edge, so it is the
    // one part of a frame that does not move between the pair. Cropping it off
    // is necessary because the terminal tile skips its exclusion mask and would
    // otherwise paint that chrome over the placeholder fills an earlier tile
    // left in a masked band. Only the chrome on the pair's *overlapping* side
    // may go: the strips are measured from the axis origin regardless of travel
    // direction, so the current tile overlaps its predecessor at its origin
    // edge when travelling Forward and at its far edge when travelling
    // Backward. Cropping the other side would remove content no other tile
    // covers, punching a hole in the canvas. The first tile keeps its own
    // leading edge: no pair precedes it, and at the page top the header really
    // is the document's first rows.
    let axis_length = movement.axis.length(&frame.image);
    let (previous_strip, current_strip) = match movement.direction {
      Direction::Backward => (regions.leading_strip, regions.trailing_strip),
      Direction::Forward => (regions.trailing_strip, regions.leading_strip),
    };
    let previous_crop = TRAILING_EDGE_CROP.max(previous_strip);
    let current_crop = current_strip.min(
      axis_length
        .saturating_sub(shift)
        .saturating_sub(previous_crop)
        .saturating_sub(MINIMUM_PAIR_OVERLAP),
    );
    let mut crop = Crop::default();
    let previous = tiles
      .last_mut()
      .ok_or_else(|| "The scrolling capture produced no frames".to_owned())?;
    match (movement.axis, movement.direction) {
      (Axis::Horizontal, Direction::Backward) => {
        previous.crop.left = previous_crop;
        crop.right = current_crop;
      }
      (Axis::Horizontal, Direction::Forward) => {
        previous.crop.right = previous_crop;
        crop.left = current_crop;
      }
      (Axis::Vertical, Direction::Backward) => {
        previous.crop.top = previous_crop;
        crop.bottom = current_crop;
      }
      (Axis::Vertical, Direction::Forward) => {
        previous.crop.bottom = previous_crop;
        crop.top = current_crop;
      }
    }
    tiles.push(Tile {
      crop,
      fixed_bands: Some(regions.bands),
      image: frame.image,
      x,
      y,
    });
  }

  compose(tiles)
}

#[cfg(test)]
#[path = "canvas_tests.rs"]
mod tests;
