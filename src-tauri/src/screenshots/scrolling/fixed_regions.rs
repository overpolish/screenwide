// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Axis, Direction};
use crate::{
  image_analysis::{detect_background_where, ImageRegion, ImageView},
  screenshots::CapturedImage,
};

const BAND_COUNT: usize = 32;
const MINIMUM_STATIC_SAMPLES: u32 = 24;
const SAMPLE_STEP: u32 = 8;
const STATIC_LUMA_TOLERANCE: i32 = 8;
/// A position counts as viewport-static once this share of its samples held
/// still; anti-aliased text over a translucent bar leaves a few pixels moving.
const STATIC_STRIP_PERCENTAGE: u32 = 90;
/// Chrome can never plausibly own most of the viewport, and an unbounded strip
/// on a barely changed pair would crop away real page content.
const STATIC_STRIP_CAP_NUMERATOR: u32 = 2;
const STATIC_STRIP_CAP_DENOMINATOR: u32 = 5;

pub(super) struct FixedBands {
  axis: Axis,
  backgrounds: Vec<Option<[u8; 4]>>,
  right_edge_start: Option<usize>,
}

/// Everything one frame pair reveals about viewport-fixed chrome: the bands
/// across the scroll axis, plus the depth of any static strip at either end of
/// the axis itself (a sticky header, a sticky footer or toolbar).
pub(super) struct FixedRegions {
  pub(super) bands: FixedBands,
  pub(super) leading_strip: u32,
  pub(super) trailing_strip: u32,
}

/// Width of one detection band: the granularity at which the left boundary of
/// a right-edge strip is known.
pub(super) fn detection_band_width(width: u32) -> u32 {
  width / BAND_COUNT as u32
}

fn pixel(image: &CapturedImage, x: u32, y: u32) -> [u8; 4] {
  let offset = ((y * image.width + x) * 4) as usize;
  image.rgba[offset..offset + 4]
    .try_into()
    .unwrap_or_default()
}

fn luma(colour: [u8; 4]) -> i32 {
  (i32::from(colour[0]) * 54 + i32::from(colour[1]) * 183 + i32::from(colour[2]) * 19) >> 8
}

fn gradient(image: &CapturedImage, x: u32, y: u32) -> i32 {
  let value = luma(pixel(image, x, y));
  let right = luma(pixel(image, (x + 1).min(image.width - 1), y));
  let below = luma(pixel(image, x, (y + 1).min(image.height - 1)));
  (right - value).abs() + (below - value).abs()
}

fn coordinates(axis: Axis, along: u32, across: u32) -> (u32, u32) {
  match axis {
    Axis::Horizontal => (along, across),
    Axis::Vertical => (across, along),
  }
}

/// Votes for the most common colour of a band, optionally counting only the
/// pixels that held still between the pair.
fn vote(
  previous: Option<&CapturedImage>,
  current: &CapturedImage,
  axis: Axis,
  start: u32,
  end: u32,
) -> Option<[u8; 4]> {
  let region = match axis {
    Axis::Horizontal => ImageRegion {
      height: end.saturating_sub(start),
      width: current.width,
      x: 0,
      y: start,
    },
    Axis::Vertical => ImageRegion {
      height: current.height,
      width: end.saturating_sub(start),
      x: start,
      y: 0,
    },
  };
  let sample = detect_background_where(
    ImageView {
      height: current.height,
      rgba: &current.rgba,
      width: current.width,
    },
    region,
    SAMPLE_STEP,
    |x, y| {
      previous.is_none_or(|previous| {
        x < previous.width
          && y < previous.height
          && (luma(pixel(previous, x, y)) - luma(pixel(current, x, y))).abs()
            <= STATIC_LUMA_TOLERANCE
      })
    },
  )?;
  if previous.is_some() && sample.samples < MINIMUM_STATIC_SAMPLES {
    return None;
  }
  Some(sample.colour)
}

/// The fill stands in for the rail's own background, so only pixels that were
/// viewport-static across the pair may vote: an ad card animating in, or page
/// content scrolling through the band, is not part of the rail and a bright
/// transient would otherwise paint a bright strip down a dark rail. A band
/// with too few still pixels to trust falls back to voting over all of them.
fn dominant_colour(
  previous: &CapturedImage,
  current: &CapturedImage,
  axis: Axis,
  start: u32,
  end: u32,
) -> [u8; 4] {
  vote(Some(previous), current, axis, start, end)
    .or_else(|| vote(None, current, axis, start, end))
    .unwrap_or([0, 0, 0, 255])
}

/// Whether one row (vertical axis) or column (horizontal axis) is unchanged
/// between the pair at *identical viewport coordinates* - the signature of
/// chrome that stays put while the document scrolls underneath it.
fn position_is_static(
  previous: &CapturedImage,
  current: &CapturedImage,
  axis: Axis,
  along: u32,
) -> bool {
  let across_size = axis.cross_length(previous).min(axis.cross_length(current));
  let mut samples = 0_u32;
  let mut still = 0_u32;
  let mut across = 0;
  while across < across_size {
    let (x, y) = coordinates(axis, along, across);
    samples += 1;
    if (luma(pixel(previous, x, y)) - luma(pixel(current, x, y))).abs() <= STATIC_LUMA_TOLERANCE {
      still += 1;
    }
    across += SAMPLE_STEP;
  }
  samples > 0 && still * 100 >= samples * STATIC_STRIP_PERCENTAGE
}

/// Measures how deep the viewport-static strips reach in from each end of the
/// scroll axis.
fn static_strips(previous: &CapturedImage, current: &CapturedImage, axis: Axis) -> (u32, u32) {
  let length = axis.length(previous).min(axis.length(current));
  if length == 0 {
    return (0, 0);
  }
  let cap = length / STATIC_STRIP_CAP_DENOMINATOR * STATIC_STRIP_CAP_NUMERATOR;
  let mut leading = 0;
  while leading < cap && position_is_static(previous, current, axis, leading) {
    leading += 1;
  }
  let mut trailing = 0;
  while trailing < cap && position_is_static(previous, current, axis, length - 1 - trailing) {
    trailing += 1;
  }
  (leading, trailing)
}

/// Reduces the classified bands to the rail at the leading edge, reporting
/// where the trailing run of fixed bands began.
///
/// A fixed rail or sidebar is edge-connected, while an interior band can match
/// poorly because of sparse text, animations or lazy content; masking those
/// would cut vertical or horizontal holes in the page. The trailing run goes
/// too, because an overlay scrollbar thumb makes the right-edge band read as
/// chrome on the pairs it crosses: painting it over erases the page behind it -
/// a source-control panel's status letters - in the stripes that flipping
/// classification draws. Composition rebuilds that strip from what the covering
/// tiles agree on instead. A frame with no unfixed band at all is ambiguous and
/// keeps every one of them.
fn keep_leading_rail(backgrounds: &mut [Option<[u8; 4]>]) -> Option<usize> {
  let prefix_end = backgrounds.iter().position(Option::is_none)?;
  let suffix_start = backgrounds.iter().rposition(Option::is_none)? + 1;
  for background in &mut backgrounds[prefix_end..] {
    *background = None;
  }
  (suffix_start < backgrounds.len()).then_some(suffix_start)
}

pub(super) fn detect(
  previous: &CapturedImage,
  current: &CapturedImage,
  axis: Axis,
  direction: Direction,
  shift: u32,
) -> FixedRegions {
  let across_size = axis.cross_length(previous);
  let overlap = axis.length(previous).saturating_sub(shift);
  let mut backgrounds = Vec::with_capacity(BAND_COUNT);
  for band in 0..BAND_COUNT {
    let start = (across_size as usize * band / BAND_COUNT) as u32;
    let end = (across_size as usize * (band + 1) / BAND_COUNT) as u32;
    let mut features = 0_u32;
    let mut difference = 0_u64;
    let mut along = 0;
    while along < overlap {
      let mut across = start;
      while across < end {
        let ((previous_x, previous_y), (current_x, current_y)) =
          axis.mapped_points(direction, shift, along, across);
        if gradient(previous, previous_x, previous_y).max(gradient(current, current_x, current_y))
          >= 10
        {
          features += 1;
          difference += u64::from(
            (luma(pixel(previous, previous_x, previous_y))
              - luma(pixel(current, current_x, current_y)))
            .unsigned_abs(),
          );
        }
        across += SAMPLE_STEP;
      }
      along += SAMPLE_STEP;
    }
    let error = difference as f64 / f64::from(features.max(1));
    let is_fixed = features >= 8 && error > 32.0;
    backgrounds.push(is_fixed.then(|| dominant_colour(previous, current, axis, start, end)));
  }
  let (leading_strip, trailing_strip) = static_strips(previous, current, axis);
  FixedRegions {
    bands: FixedBands::new(axis, backgrounds),
    leading_strip,
    trailing_strip,
  }
}

impl FixedBands {
  fn new(axis: Axis, mut backgrounds: Vec<Option<[u8; 4]>>) -> Self {
    let right_edge_start = keep_leading_rail(&mut backgrounds);
    Self {
      axis,
      backgrounds,
      right_edge_start,
    }
  }

  /// Builds the bands `detect` would return for these classifications.
  #[cfg(test)]
  pub(super) fn masking(axis: Axis, masked: &[(usize, [u8; 4])]) -> Self {
    let mut backgrounds = vec![None; BAND_COUNT];
    for (band, colour) in masked {
      backgrounds[*band] = Some(*colour);
    }
    Self::new(axis, backgrounds)
  }

  pub(super) fn background(&self, x: u32, y: u32, image: &CapturedImage) -> Option<[u8; 4]> {
    let (across, size) = match self.axis {
      Axis::Horizontal => (y, image.height),
      Axis::Vertical => (x, image.width),
    };
    let band = ((across as usize * BAND_COUNT) / size.max(1) as usize).min(BAND_COUNT - 1);
    self.backgrounds[band]
  }

  /// Width of independently moving chrome connected to the right viewport edge
  /// - the strip `keep_leading_rail` dropped, for reconstruction to rebuild.
  pub(super) fn right_edge_width(&self, image: &CapturedImage) -> u32 {
    if self.axis != Axis::Vertical {
      return 0;
    }
    let Some(start) = self.right_edge_start else {
      return 0;
    };
    let first_pixel = (start * image.width as usize).div_ceil(self.backgrounds.len()) as u32;
    image.width.saturating_sub(first_pixel)
  }
}

#[cfg(test)]
#[path = "fixed_regions_tests.rs"]
mod tests;
