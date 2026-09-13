// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Picks the first preference a device can actually deliver.
///
/// Each range is `(min, max)` fps. A preference is deliverable when some range
/// brackets it; the earliest such preference wins. When the device brackets
/// none of them the leading preference is clamped into the closest range, which
/// is the historical behaviour for cameras that advertise one fixed cadence.
#[cfg(any(test, target_os = "macos"))]
pub(super) fn choose_fps(ranges: &[(f64, f64)], preferred: &[u32]) -> u32 {
  for candidate in preferred {
    let target = f64::from(*candidate);
    if ranges
      .iter()
      .any(|(min, max)| *min <= target && *max >= target)
    {
      return *candidate;
    }
  }
  let requested = leading_fps(preferred);
  let target = f64::from(requested);
  ranges
    .iter()
    .map(|(min, max)| {
      if target < *min {
        min.ceil().max(1.0) as u32
      } else {
        max.floor().max(1.0) as u32
      }
    })
    .min_by_key(|fps| fps.abs_diff(requested))
    .unwrap_or(requested)
}

pub(super) fn preferred_mode(
  modes: &[(u32, u32, u32)],
  requested_fps: u32,
) -> Option<(u32, u32, u32)> {
  modes.iter().copied().min_by_key(|(width, height, fps)| {
    let aspect_error = u64::from(*width)
      .saturating_mul(9)
      .abs_diff(u64::from(*height).saturating_mul(16))
      .saturating_mul(1_000_000)
      / u64::from(*height).max(1);
    (
      fps.abs_diff(requested_fps),
      aspect_error,
      std::cmp::Reverse(u64::from(*width) * u64::from(*height)),
    )
  })
}

pub(super) fn sort_camera_modes(modes: &mut [(u32, u32, u32)], requested_fps: u32) {
  modes.sort_by_key(|(width, height, fps)| {
    let orientation = match width.cmp(height) {
      std::cmp::Ordering::Greater => 0,
      std::cmp::Ordering::Equal => 1,
      std::cmp::Ordering::Less => 2,
    };
    (
      std::cmp::Reverse((*width).max(*height)),
      orientation,
      std::cmp::Reverse(u64::from(*width) * u64::from(*height)),
      fps.abs_diff(requested_fps),
    )
  });
}
