// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy)]
pub(crate) struct ImageView<'a> {
  pub(crate) height: u32,
  pub(crate) rgba: &'a [u8],
  pub(crate) width: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct ImageRegion {
  pub(crate) height: u32,
  pub(crate) width: u32,
  pub(crate) x: u32,
  pub(crate) y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BackgroundSample {
  pub(crate) colour: [u8; 4],
  pub(crate) confidence: f32,
  pub(crate) samples: u32,
}

#[derive(Clone, Default)]
struct ColourBucket {
  count: u32,
  sums: [u64; 4],
}

/// The unfiltered form of [`detect_background_where`]. Every production caller
/// now passes a predicate, so this stays as the plain entry point the tests
/// exercise rather than dead weight in a release build.
#[cfg(test)]
pub(crate) fn detect_background(
  image: ImageView<'_>,
  region: ImageRegion,
  sample_step: u32,
) -> Option<BackgroundSample> {
  detect_background_where(image, region, sample_step, |_, _| true)
}

/// Finds the representative colour of the most common 4-bit-per-channel RGB
/// bucket in a region. The predicate lets pair-aware callers exclude pixels
/// that moved, while ordinary callers can pass one that accepts everything.
pub(crate) fn detect_background_where(
  image: ImageView<'_>,
  region: ImageRegion,
  sample_step: u32,
  mut include: impl FnMut(u32, u32) -> bool,
) -> Option<BackgroundSample> {
  let expected_length = image
    .width
    .checked_mul(image.height)?
    .checked_mul(4)?
    .try_into()
    .ok()?;
  if image.width == 0 || image.height == 0 || image.rgba.len() < expected_length {
    return None;
  }
  let end_x = region.x.checked_add(region.width)?.min(image.width);
  let end_y = region.y.checked_add(region.height)?.min(image.height);
  if region.x >= end_x || region.y >= end_y {
    return None;
  }

  let mut buckets = vec![ColourBucket::default(); 1 << 12];
  let mut samples = 0_u32;
  let step = sample_step.max(1) as usize;
  for y in (region.y..end_y).step_by(step) {
    for x in (region.x..end_x).step_by(step) {
      if !include(x, y) {
        continue;
      }
      let offset = ((y * image.width + x) * 4) as usize;
      let colour: [u8; 4] = image.rgba[offset..offset + 4].try_into().ok()?;
      let key = (usize::from(colour[0] >> 4) << 8)
        | (usize::from(colour[1] >> 4) << 4)
        | usize::from(colour[2] >> 4);
      let bucket = &mut buckets[key];
      bucket.count += 1;
      for (sum, channel) in bucket.sums.iter_mut().zip(colour) {
        *sum += u64::from(channel);
      }
      samples += 1;
    }
  }

  let bucket = buckets.into_iter().max_by_key(|bucket| bucket.count)?;
  if bucket.count == 0 {
    return None;
  }
  Some(BackgroundSample {
    colour: bucket.sums.map(|sum| (sum / u64::from(bucket.count)) as u8),
    confidence: bucket.count as f32 / samples as f32,
    samples,
  })
}

#[cfg(test)]
#[path = "image_analysis_tests.rs"]
mod tests;
