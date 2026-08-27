// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use rayon::prelude::*;

use super::super::{Axis, Direction};
use crate::screenshots::CapturedImage;

const ALIGNMENT_BANDS: usize = 8;
const ALIGNMENT_SEGMENTS: usize = 4;
const MIN_SEGMENT_RUN: usize = 3;
const CHANGE_SAMPLE_STEP: u32 = 8;

/// Per-frame luma and gradient planes.
///
/// Every candidate shift rescores the same pixels, so recomputing luma from
/// RGBA inside the sample loop made the search cost dominated by arithmetic on
/// values that never change. Both planes are built once per captured frame and
/// then shared by the two pairs that frame takes part in.
pub(super) struct Planes {
  gradient: Vec<u8>,
  pub(super) height: u32,
  luma: Vec<u8>,
  pub(super) width: u32,
}

impl Planes {
  pub(super) fn new(image: &CapturedImage) -> Self {
    let width = image.width;
    let height = image.height;
    let count = width as usize * height as usize;

    let mut luma = vec![0_u8; count];
    luma
      .par_iter_mut()
      .zip(image.rgba.par_chunks_exact(4))
      .for_each(|(value, colour)| {
        let red = u32::from(colour[0]);
        let green = u32::from(colour[1]);
        let blue = u32::from(colour[2]);
        *value = ((red * 54 + green * 183 + blue * 19) >> 8) as u8;
      });

    let mut gradient = vec![0_u8; count];
    if width > 0 && height > 0 {
      let stride = width as usize;
      let last_column = (width - 1) as usize;
      let last_row = height - 1;
      gradient
        .par_chunks_mut(stride)
        .enumerate()
        .for_each(|(row, output)| {
          let above = row * stride;
          let below = (row + 1).min(last_row as usize) * stride;
          for (column, value) in output.iter_mut().enumerate() {
            let here = i32::from(luma[above + column]);
            let right = i32::from(luma[above + (column + 1).min(last_column)]);
            let under = i32::from(luma[below + column]);
            // Clamped because the only consumer is a `>= 10` feature gate, so
            // saturating well above the threshold cannot change an outcome.
            *value = ((right - here).abs() + (under - here).abs()).min(255) as u8;
          }
        });
    }

    Self {
      gradient,
      height,
      luma,
      width,
    }
  }

  pub(super) fn extents(&self, axis: Axis) -> (u32, u32) {
    match axis {
      Axis::Horizontal => (self.width, self.height),
      Axis::Vertical => (self.height, self.width),
    }
  }
}

pub(super) fn alignment_error(
  previous: &Planes,
  current: &Planes,
  axis: Axis,
  direction: Direction,
  shift: u32,
  sample_step: u32,
) -> Option<f64> {
  let (along_size, across_size) = previous.extents(axis);
  let overlap = along_size.checked_sub(shift)?;
  if overlap < 8 || across_size < 8 {
    return None;
  }

  let stride = previous.width as usize;
  let margin = (across_size / 20).max(2).min(across_size / 3);
  let mut differences = [[0_u64; ALIGNMENT_BANDS]; ALIGNMENT_SEGMENTS];
  let mut samples = [[0_u64; ALIGNMENT_BANDS]; ALIGNMENT_SEGMENTS];
  let mut along = 1_u32;
  while along + 1 < overlap {
    let segment =
      ((along as usize * ALIGNMENT_SEGMENTS) / overlap as usize).min(ALIGNMENT_SEGMENTS - 1);
    let mut across = margin.max(1);
    while across + margin + 1 < across_size {
      let ((previous_x, previous_y), (current_x, current_y)) =
        axis.mapped_points(direction, shift, along, across);
      let previous_index = previous_y as usize * stride + previous_x as usize;
      let current_index = current_y as usize * stride + current_x as usize;
      let previous_gradient = i32::from(previous.gradient[previous_index]);
      let current_gradient = i32::from(current.gradient[current_index]);
      // Fixed headers, sidebars and overlays remain at the same viewport
      // coordinate. They are not evidence for document displacement and can
      // dominate a narrow overlap, so only score samples whose viewport pixel
      // actually changed between frames.
      let viewport_change =
        (i32::from(previous.luma[current_index]) - i32::from(current.luma[current_index])).abs();
      if previous_gradient.max(current_gradient) >= 10 && viewport_change > 8 {
        let band =
          ((across as usize * ALIGNMENT_BANDS) / across_size as usize).min(ALIGNMENT_BANDS - 1);
        differences[segment][band] += u64::from(
          (i32::from(previous.luma[previous_index]) - i32::from(current.luma[current_index]))
            .unsigned_abs(),
        );
        samples[segment][band] += 1;
      }
      across += sample_step;
    }
    along += sample_step;
  }

  if samples.iter().flatten().sum::<u64>() < 24 {
    return None;
  }
  // Chrome that *moves* between the two frames passes the viewport-change gate
  // above and is then scored as if it were document. A banner dismissed on the
  // first scroll does exactly that to the whole sticky layout anchored under
  // it, and the contamination lands in a cross: the header spans every
  // cross-axis band but only the leading rows, while the sidebar that slid up
  // with it spans every along-axis segment but only the leading bands. Neither
  // axis alone can excise a cross, so the grid is collapsed jointly - best
  // three bands within the best run of segments - and only a region excluded on
  // both axes at once survives, which is precisely the clean document.
  //
  // The segment run must be contiguous. Moving chrome sits at an edge of the
  // overlap (or, with both edges sticky, spans the middle), whereas vertically
  // periodic content alternates between agreeing and disagreeing segments;
  // picking segments by rank would hand such an alias the agreeing ones and a
  // score of zero. Every contiguous run mixes the two, so an alias cannot
  // escape, and the full-overlap run reproduces the plain banded statistic for
  // the ordinary page that has no moving chrome at all.
  //
  // The bands are chosen once, over the whole overlap, and every run is then
  // scored on that same choice. Chrome contaminating a run leaves the ranking
  // between bands intact - it lands on all of them - so the choice is as good
  // as one made per run, while re-picking the best three inside each run would
  // let a lucky quarter of a mismatching frame drift below the threshold.
  let mut total_differences = [0_u64; ALIGNMENT_BANDS];
  let mut total_samples = [0_u64; ALIGNMENT_BANDS];
  for band in 0..ALIGNMENT_BANDS {
    for segment in 0..ALIGNMENT_SEGMENTS {
      total_differences[band] += differences[segment][band];
      total_samples[band] += samples[segment][band];
    }
  }
  let mut ranked: Vec<(f64, usize)> = (0..ALIGNMENT_BANDS)
    .filter(|band| total_samples[*band] >= 8)
    .map(|band| {
      (
        total_differences[band] as f64 / total_samples[band] as f64,
        band,
      )
    })
    .collect();
  ranked.sort_by(|left, right| left.0.total_cmp(&right.0));
  ranked.truncate(3);
  if ranked.is_empty() {
    return None;
  }
  let banded = |differences: &[u64; ALIGNMENT_BANDS], samples: &[u64; ALIGNMENT_BANDS]| {
    let scored: Vec<f64> = ranked
      .iter()
      .filter(|(_, band)| samples[*band] >= 8)
      .map(|(_, band)| differences[*band] as f64 / samples[*band] as f64)
      .collect();
    (!scored.is_empty()).then(|| scored.iter().sum::<f64>() / scored.len() as f64)
  };

  let mut best_error: Option<f64> = None;
  for start in 0..ALIGNMENT_SEGMENTS {
    let mut run_differences = [0_u64; ALIGNMENT_BANDS];
    let mut run_samples = [0_u64; ALIGNMENT_BANDS];
    for end in start..ALIGNMENT_SEGMENTS {
      for band in 0..ALIGNMENT_BANDS {
        run_differences[band] += differences[end][band];
        run_samples[band] += samples[end][band];
      }
      if end + 1 - start < MIN_SEGMENT_RUN {
        continue;
      }
      let Some(error) = banded(&run_differences, &run_samples) else {
        continue;
      };
      if end + 1 - start < ALIGNMENT_SEGMENTS {
        // Dropping segments is only warranted when what was dropped looks like
        // foreign content rather than merely the worse end of one mismatch: a
        // whole threshold of separation is what a moving header shows (a
        // wholesale disagreement against a run that agrees), while a periodic
        // alias is uniformly mediocre and would otherwise buy an undeserved
        // discount by shedding its weakest quarter.
        let excluded_differences =
          std::array::from_fn(|band| total_differences[band] - run_differences[band]);
        let excluded_samples = std::array::from_fn(|band| total_samples[band] - run_samples[band]);
        match banded(&excluded_differences, &excluded_samples) {
          Some(excluded) if excluded >= error + super::MAX_ALIGNMENT_ERROR => {}
          _ => continue,
        }
      }
      best_error = Some(best_error.map_or(error, |best: f64| best.min(error)));
    }
  }
  best_error
}

pub(super) fn sampled_change(previous: &Planes, current: &Planes) -> (u64, u64) {
  let stride = previous.width as usize;
  let mut changed = 0_u64;
  let mut samples = 0_u64;
  let mut y = 0;
  while y < previous.height {
    let row = y as usize * stride;
    let mut x = 0;
    while x < previous.width {
      let index = row + x as usize;
      if (i32::from(previous.luma[index]) - i32::from(current.luma[index])).abs() > 8 {
        changed += 1;
      }
      samples += 1;
      x += CHANGE_SAMPLE_STEP;
    }
    y += CHANGE_SAMPLE_STEP;
  }
  (changed, samples)
}
