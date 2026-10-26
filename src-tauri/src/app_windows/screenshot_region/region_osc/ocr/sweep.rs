// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheen that passes over the area while Text Recognition reads it: a
//! slanted band of light crossing it from start to end, resting a moment,
//! then crossing again, so the wait reads as work under way. The band is
//! drawn by `ocr_sweep` in `osc.wgsl`; here it is placed for the moment.

use screenwide_osc::geometry::Rect;

/// How far the band leans, across per unit down: `OCR_SWEEP_SLANT`.
const SLANT: f64 = 0.35;
/// One crossing, and the rest before the next, in seconds.
const CROSSING: f64 = 1.1;
const REST: f64 = 0.4;
/// How wide the band is: a share of the area, within limits, in points.
const BAND_SHARE: f64 = 0.3;
const BAND_MIN: f64 = 48.0;
const BAND_MAX: f64 = 220.0;
/// Past this many band widths from its centre line the band has no light.
const BAND_REACH: f64 = 1.05;

/// When the crossing under way at `elapsed` seconds ends, in seconds since
/// reading began; none while the band rests between crossings.
pub(super) fn crossing_end(elapsed: f64) -> Option<f64> {
  let period = CROSSING + REST;
  let start = (elapsed.max(0.0) / period).floor() * period;
  (elapsed - start < CROSSING).then_some(start + CROSSING)
}

/// Where the band is `elapsed` seconds after reading began: the quad over
/// `region` and its uv, in band widths from where the band's centre line
/// crosses the area's middle. None while it rests off the area.
pub(super) fn band(region: Rect, elapsed: f64) -> Option<(Rect, Rect)> {
  let (width, height) = (region.size.width, region.size.height);
  if width <= 0.0 || height <= 0.0 {
    return None;
  }
  let progress = (elapsed.max(0.0) % (CROSSING + REST)) / CROSSING;
  if progress >= 1.0 {
    return None;
  }
  // Easing in and out, so the light gathers pace and settles as it leaves.
  let eased = progress * progress * (3.0 - 2.0 * progress);
  let band = (width * BAND_SHARE).clamp(BAND_MIN, BAND_MAX);
  let reach = band * BAND_REACH + SLANT * height / 2.0;
  let from = region.origin.x - reach;
  let to = region.right() + reach;
  let centre = from + (to - from) * eased;
  let middle = region.origin.y + height / 2.0;
  Some((
    region,
    Rect::from_xywh(
      (region.origin.x - centre) / band,
      (region.origin.y - middle) / band,
      width / band,
      height / band,
    ),
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  const AREA: Rect = Rect::from_xywh(100.0, 100.0, 400.0, 200.0);

  /// Where the band's centre line crosses the area's middle, in points.
  fn centre(uv: Rect) -> f64 {
    let band = (AREA.size.width * BAND_SHARE).clamp(BAND_MIN, BAND_MAX);
    AREA.origin.x - uv.origin.x * band
  }

  #[test]
  fn the_band_crosses_from_beyond_the_start_to_beyond_the_end() {
    let (_, start) = band(AREA, 0.0).unwrap();
    let (_, middle) = band(AREA, CROSSING / 2.0).unwrap();
    let (_, end) = band(AREA, CROSSING * 0.999).unwrap();
    assert!(centre(start) < AREA.origin.x);
    assert!((centre(middle) - 300.0).abs() < 1e-9);
    assert!(centre(end) > AREA.right());
  }

  #[test]
  fn the_band_rests_off_the_area_before_crossing_again() {
    assert!(band(AREA, CROSSING + REST / 2.0).is_none());
    assert!(band(AREA, CROSSING + REST + 0.01).is_some());
  }

  #[test]
  fn a_crossing_under_way_ends_where_it_would_have_and_a_rest_has_none() {
    let second = CROSSING + REST;
    assert_eq!(crossing_end(0.2), Some(CROSSING));
    assert_eq!(crossing_end(second + 0.1), Some(second + CROSSING));
    assert_eq!(crossing_end(CROSSING + REST / 2.0), None);
  }

  #[test]
  fn the_uv_is_measured_in_band_widths_from_the_middle() {
    let (_, uv) = band(AREA, CROSSING / 2.0).unwrap();
    assert_eq!(uv.size.width, 400.0 / 120.0);
    assert_eq!(uv.origin.y, -100.0 / 120.0);
  }
}
