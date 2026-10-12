// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time on an edited recording's timeline: the source ranges it keeps and the
//! speed each plays at, mapping between source and output time, and the
//! easing curves effects move along it.

pub mod easing;
mod time_mapping;

pub use time_mapping::{
  ascending_source_pieces, output_at_us, output_duration_us, output_end_at_us,
  output_start_to_source_us, output_to_source_us, plays_through, rate_at,
  source_after_output_duration_us, source_before_output_duration_us, source_to_output_us,
};

/// A stretch of the source the timeline keeps, the rate it plays at, and
/// where it starts on the output.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRange {
  pub output_start_us: u64,
  pub source_end_us: u64,
  pub source_start_us: u64,
  pub playback_rate: f64,
}
