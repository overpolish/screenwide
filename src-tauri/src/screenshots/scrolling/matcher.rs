// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use rayon::prelude::*;

use super::{Axis, Direction};
use crate::screenshots::CapturedImage;

mod planes;

use planes::{alignment_error, sampled_change, Planes};

const ALIGNMENT_SAMPLE_STEP: u32 = 8;
const MAX_ALIGNMENT_ERROR: f64 = 24.0;
const REFINE_SAMPLE_STEP: u32 = 4;
const SWEEP_COARSE_STEP: u32 = 4;
const WINDOW_COARSE_STEP: u32 = 1;

pub(super) struct ShiftMatch {
  pub(super) shift: Option<u32>,
}

/// A captured frame paired with the planes derived from it.
///
/// Cloning shares both, so a frame that is the `current` of one pair and the
/// `previous` of the next is only ever analysed once.
#[derive(Clone)]
pub(super) struct MatchFrame {
  pub(super) image: Arc<CapturedImage>,
  planes: Arc<Planes>,
}

impl MatchFrame {
  pub(super) fn new(image: Arc<CapturedImage>) -> Self {
    let planes = Arc::new(Planes::new(&image));
    Self { image, planes }
  }
}

pub(super) fn frames_are_same(previous: &MatchFrame, current: &MatchFrame) -> bool {
  let previous = previous.planes.as_ref();
  let current = current.planes.as_ref();
  if previous.width != current.width || previous.height != current.height {
    return false;
  }
  let (changed, samples) = sampled_change(previous, current);
  samples == 0 || changed * 100 <= samples * 2
}

fn coarse_scores(
  previous: &Planes,
  current: &Planes,
  axis: Axis,
  direction: Direction,
  expected: u32,
  length: u32,
  candidates: &[u32],
) -> Vec<(f64, u32)> {
  // Candidates keep their ascending order through the parallel map, so the
  // stable sort below leaves the ranking identical to a serial sweep.
  let mut scores: Vec<(f64, u32)> = candidates
    .par_iter()
    .filter_map(|&shift| {
      alignment_error(
        previous,
        current,
        axis,
        direction,
        shift,
        ALIGNMENT_SAMPLE_STEP,
      )
      .map(|error| {
        let proximity = f64::from(shift.abs_diff(expected)) / f64::from(length.max(1));
        (error + proximity * 1.5, shift)
      })
    })
    .collect();
  scores.sort_by(|left, right| left.0.total_cmp(&right.0));
  scores
}

#[allow(clippy::too_many_arguments)]
fn refine_candidates(
  previous: &Planes,
  current: &Planes,
  axis: Axis,
  direction: Direction,
  expected: u32,
  length: u32,
  maximum: u32,
  coarse: &[(f64, u32)],
  radius: u32,
) -> Option<(f64, f64, u32)> {
  let mut refined = Vec::new();
  for &(_, candidate) in coarse.iter().take(4) {
    let start = candidate.saturating_sub(radius);
    let end = (candidate + radius).min(maximum);
    for shift in start.max(1)..=end {
      if let Some(error) = alignment_error(
        previous,
        current,
        axis,
        direction,
        shift,
        REFINE_SAMPLE_STEP,
      ) {
        let proximity = f64::from(shift.abs_diff(expected)) / f64::from(length.max(1));
        refined.push((error + proximity, error, shift));
      }
    }
  }
  refined.sort_by(|left, right| left.0.total_cmp(&right.0));
  refined.first().copied()
}

pub(super) fn match_shift(
  previous: &MatchFrame,
  current: &MatchFrame,
  axis: Axis,
  direction: Direction,
  expected: u32,
) -> ShiftMatch {
  let previous_planes = previous.planes.as_ref();
  let current_planes = current.planes.as_ref();
  if previous_planes.width != current_planes.width
    || previous_planes.height != current_planes.height
    || previous_planes.width < 16
    || previous_planes.height < 16
  {
    return ShiftMatch { shift: None };
  }
  if frames_are_same(previous, current) {
    return ShiftMatch { shift: None };
  }

  let (length, _) = previous_planes.extents(axis);
  let maximum = (length * 85 / 100).max(2);

  // The scroll we asked for is a strong prior: a dense search of its
  // neighbourhood almost always settles the pair, and only the rare page that
  // scrolls by an unrelated amount pays for the whole sweep.
  let window = (length / 8).max(16);
  let window_start = expected.saturating_sub(window).max(2);
  let window_end = (expected + window).min(maximum);
  let window_candidates: Vec<u32> = if window_start <= window_end {
    (window_start..=window_end)
      .step_by(WINDOW_COARSE_STEP as usize)
      .collect()
  } else {
    Vec::new()
  };
  let windowed = refine_candidates(
    previous_planes,
    current_planes,
    axis,
    direction,
    expected,
    length,
    maximum,
    &coarse_scores(
      previous_planes,
      current_planes,
      axis,
      direction,
      expected,
      length,
      &window_candidates,
    ),
    WINDOW_COARSE_STEP,
  );
  if let Some((_, error, best_shift)) = windowed {
    if error <= MAX_ALIGNMENT_ERROR {
      return ShiftMatch {
        shift: Some(best_shift),
      };
    }
  }

  let sweep_candidates: Vec<u32> = (2..=maximum).step_by(SWEEP_COARSE_STEP as usize).collect();
  let swept = refine_candidates(
    previous_planes,
    current_planes,
    axis,
    direction,
    expected,
    length,
    maximum,
    &coarse_scores(
      previous_planes,
      current_planes,
      axis,
      direction,
      expected,
      length,
      &sweep_candidates,
    ),
    SWEEP_COARSE_STEP,
  );
  let Some((_, error, best_shift)) = swept.or(windowed) else {
    return ShiftMatch { shift: None };
  };
  ShiftMatch {
    shift: (error <= MAX_ALIGNMENT_ERROR).then_some(best_shift),
  }
}

pub(super) fn find_shift(
  previous: &MatchFrame,
  current: &MatchFrame,
  axis: Axis,
  direction: Direction,
  expected: u32,
) -> Option<u32> {
  match_shift(previous, current, axis, direction, expected).shift
}

#[cfg(test)]
#[path = "matcher_tests.rs"]
mod tests;
