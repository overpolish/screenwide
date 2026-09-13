// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn source_to_output_us(ranges: &[TimelineRange], source_us: u64) -> Option<u64> {
  let range = ranges.iter().find(|range| {
    source_us >= range.source_start_us
      && (source_us < range.source_end_us
        || source_us == range.source_end_us && range.source_end_us == range.source_start_us)
  })?;
  Some(range.output_start_us.saturating_add(
    ((source_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate).round() as u64,
  ))
}

pub(super) fn output_to_source_us(ranges: &[TimelineRange], output_us: u64) -> Option<u64> {
  for range in ranges {
    let output_end_us = range.output_start_us.saturating_add(
      ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
        .round() as u64,
    );
    if output_us <= output_end_us {
      return Some(range.source_start_us.saturating_add(
        ((output_us.saturating_sub(range.output_start_us)) as f64 * range.playback_rate).round()
          as u64,
      ));
    }
  }
  ranges.last().map(|range| range.source_end_us)
}

/// Converts a source-time animation anchor plus an output-time duration back
/// into a source coordinate. Timed lanes can map that coordinate normally and
/// still match animations whose duration must not stretch with playback rate.
pub(crate) fn source_after_output_duration_us(
  ranges: Option<&[TimelineRange]>,
  anchor_us: u64,
  duration_us: u64,
) -> Option<u64> {
  let Some(ranges) = ranges else {
    return Some(anchor_us.saturating_add(duration_us));
  };
  let output_anchor_us = source_to_output_us(ranges, anchor_us)?;
  output_to_source_us(ranges, output_anchor_us.saturating_add(duration_us))
}

/// The reverse of [`source_after_output_duration_us`]: the source coordinate
/// that lies an output-time duration BEFORE the anchor, so a fixed-length
/// animation can be scheduled to finish exactly at the anchor at any rate.
pub(crate) fn source_before_output_duration_us(
  ranges: Option<&[TimelineRange]>,
  anchor_us: u64,
  duration_us: u64,
) -> Option<u64> {
  let Some(ranges) = ranges else {
    return Some(anchor_us.saturating_sub(duration_us));
  };
  let output_anchor_us = source_to_output_us(ranges, anchor_us)?;
  output_to_source_us(ranges, output_anchor_us.saturating_sub(duration_us))
}
