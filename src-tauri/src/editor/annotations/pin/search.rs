// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::Rect;
use super::luma::Pyramid;

/// The longest side a template is cut at, in pixels of the level it is cut
/// from: coarse enough that a search over a wide window stays cheap, fine
/// enough that a line of text is still more than a smear.
const TEMPLATE_SIDE: f32 = 28.0;
/// The shortest side worth matching.
const MIN_SIDE: usize = 6;
/// The weakest normalized correlation taken as the target.
const MIN_SCORE: f32 = 0.8;
/// How far ahead of any other place the best one must score. A list's rows
/// or a grid's tiles look alike, and a target taken from the wrong one would
/// be followed with full confidence.
const MARGIN: f32 = 0.04;

/// How many sums the correlation keeps side by side.
const LANES: usize = 8;
/// The target's appearance at one pyramid level, with its mean taken out.
#[derive(Clone, Debug)]
pub(crate) struct Template {
  level: usize,
  width: usize,
  height: usize,
  data: Vec<f32>,
  norm: f32,
}

impl Template {
  /// `region` of `pyramid`, at the finest level where it fits
  /// [`TEMPLATE_SIDE`]. `None` where the region is not wholly on the frame or
  /// is flat.
  pub(crate) fn cut(pyramid: &Pyramid, region: &Rect) -> Option<Self> {
    let frame = Rect {
      x0: 0.0,
      y0: 0.0,
      x1: pyramid.width(),
      y1: pyramid.height(),
    };
    if region.intersect(&frame) != *region {
      return None;
    }
    let longest = region.width().max(region.height());
    let mut level = 0;
    while level + 1 < pyramid.levels.len() && longest / (1 << level) as f32 > TEMPLATE_SIDE {
      level += 1;
    }
    let scale = (1 << level) as f32;
    let plane = &pyramid.levels[level];
    let (x0, y0) = (
      (region.x0 / scale).round() as usize,
      (region.y0 / scale).round() as usize,
    );
    let width = ((region.width() / scale).round() as usize).min(plane.width.saturating_sub(x0));
    let height = ((region.height() / scale).round() as usize).min(plane.height.saturating_sub(y0));
    if width < MIN_SIDE || height < MIN_SIDE {
      return None;
    }
    let mut data = Vec::with_capacity(width * height);
    for y in 0..height {
      data.extend_from_slice(&plane.data[(y0 + y) * plane.width + x0..][..width]);
    }
    let mean = data.iter().sum::<f32>() / data.len() as f32;
    let mut norm = 0.0;
    for value in &mut data {
      *value -= mean;
      norm += *value * *value;
    }
    let norm = norm.sqrt();
    // A standard deviation under four brightness steps is flat colour.
    (norm > 4.0 * ((width * height) as f32).sqrt()).then_some(Self {
      level,
      width,
      height,
      data,
      norm,
    })
  }

  /// Where the template matches best within `radius` tracking pixels of
  /// `around`, as the matched centre in tracking pixels and its normalized
  /// correlation. `None` where nothing matches well or two places match
  /// about as well.
  pub(crate) fn find(
    &self,
    pyramid: &Pyramid,
    around: [f32; 2],
    radius: f32,
  ) -> Option<([f32; 2], f32)> {
    let scale = (1 << self.level) as f32;
    let plane = &pyramid.levels[self.level];
    let reach = (radius / scale).ceil() as isize;
    let (cx, cy) = (
      (around[0] / scale - 0.5 * self.width as f32).round() as isize,
      (around[1] / scale - 0.5 * self.height as f32).round() as isize,
    );
    let max_x = plane.width as isize - self.width as isize;
    let max_y = plane.height as isize - self.height as isize;
    let (x_from, x_to) = ((cx - reach).max(0), (cx + reach).min(max_x));
    let (y_from, y_to) = ((cy - reach).max(0), (cy + reach).min(max_y));
    if x_from > x_to || y_from > y_to {
      return None;
    }
    let columns = (x_to - x_from + 1) as usize;
    let rows = (y_to - y_from + 1) as usize;
    let mut scores = vec![-1.0_f32; columns * rows];
    let count = (self.width * self.height) as f32;
    for row in 0..rows {
      let y = (y_from as usize) + row;
      for column in 0..columns {
        let x = (x_from as usize) + column;
        let (mut sum, mut squares, mut dot) =
          ([0.0_f32; LANES], [0.0_f32; LANES], [0.0_f32; LANES]);
        for ty in 0..self.height {
          let window = &plane.data[(y + ty) * plane.width + x..][..self.width];
          let template = &self.data[ty * self.width..][..self.width];
          // Independent lanes, so the sums vectorize: a float sum in one
          // accumulator has to be added in order.
          let (window_lanes, window_rest) = window.as_chunks::<LANES>();
          let (template_lanes, template_rest) = template.as_chunks::<LANES>();
          for (values, weights) in window_lanes.iter().zip(template_lanes) {
            for lane in 0..LANES {
              sum[lane] += values[lane];
              squares[lane] += values[lane] * values[lane];
              dot[lane] += values[lane] * weights[lane];
            }
          }
          for (value, weight) in window_rest.iter().zip(template_rest) {
            sum[0] += value;
            squares[0] += value * value;
            dot[0] += value * weight;
          }
        }
        let (sum, squares, dot) = (
          sum.iter().sum::<f32>(),
          squares.iter().sum::<f32>(),
          dot.iter().sum::<f32>(),
        );
        let variance = squares - sum * sum / count;
        if variance > 1.0 {
          scores[row * columns + column] = dot / (variance.sqrt() * self.norm);
        }
      }
    }
    let (best, best_score) = scores
      .iter()
      .enumerate()
      .max_by(|a, b| a.1.total_cmp(b.1))
      .map(|(index, score)| (index, *score))?;
    if best_score < MIN_SCORE {
      return None;
    }
    let (best_row, best_column) = (best / columns, best % columns);
    // The runner-up must sit outside the best's own peak.
    let clear_x = (self.width / 3).max(2);
    let clear_y = (self.height / 3).max(2);
    let rival = scores
      .iter()
      .enumerate()
      .filter(|(index, _)| {
        let (row, column) = (index / columns, index % columns);
        row.abs_diff(best_row) > clear_y || column.abs_diff(best_column) > clear_x
      })
      .map(|(_, score)| *score)
      .fold(-1.0_f32, f32::max);
    if best_score - rival < MARGIN {
      return None;
    }
    let x = (x_from as usize + best_column) as f32 + 0.5 * self.width as f32;
    let y = (y_from as usize + best_row) as f32 + 0.5 * self.height as f32;
    Some(([x * scale, y * scale], best_score))
  }
}
