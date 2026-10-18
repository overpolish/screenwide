// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clock an export draws on, on every platform: one frame every
//! `1 / fps` of output time, each showing the moment of the source the
//! timeline puts there. Capture may omit unchanged screen frames, but effects
//! still need an output clock: drawing only the frames captured would freeze
//! the cursor and the annotations between them. macOS reaches it through the
//! C functions at the end.

use screenwide_timeline::TimelineRange;

/// The slowest an export is drawn at, whatever the recording's own rate: the
/// cursor and the annotations move at least this smoothly.
const MIN_FRAME_RATE: f64 = 60.0;

/// How far before a stretch a restarted reader begins, so the frame showing
/// at its first moment is read too rather than the one after it.
pub(super) const REWIND_PREROLL_US: u64 = 500_000;

/// How far ahead of the frame in hand a wanted frame must be before the
/// readers start again there rather than decode their way to it. A cut this
/// long, or a gap between the stretches a size estimate draws, costs less to
/// seek over than to decode through.
const SKIP_AHEAD_US: u64 = 3_000_000;

/// The whole rate an export of a recording captured at `nominal` frames a
/// second is drawn at.
pub(super) fn frame_rate(nominal: Option<f64>) -> u32 {
  nominal
    .filter(|rate| rate.is_finite())
    .map_or(MIN_FRAME_RATE, |rate| rate.max(MIN_FRAME_RATE))
    .round() as u32
}

fn range_end_us(range: &TimelineRange) -> u64 {
  range.output_start_us
    + ((range.source_end_us - range.source_start_us) as f64 / range.playback_rate).round() as u64
}

/// How long the exported video runs: the source's own `source_duration_us`,
/// or where `ranges` lay their last stretch out.
pub(super) fn duration_us(source_duration_us: u64, ranges: &[TimelineRange]) -> u64 {
  ranges.last().map_or(source_duration_us, range_end_us)
}

/// The moment of the source shown `output_us` into the export, or `None`
/// where `ranges` leave nothing there. Without ranges the two clocks are one.
pub(super) fn source_us(output_us: u64, ranges: &[TimelineRange]) -> Option<u64> {
  if ranges.is_empty() {
    return Some(output_us);
  }
  ranges
    .iter()
    .find(|range| output_us >= range.output_start_us && output_us < range_end_us(range))
    .map(|range| {
      let source = range.source_start_us
        + ((output_us - range.output_start_us) as f64 * range.playback_rate).round() as u64;
      source.min(range.source_end_us.saturating_sub(1))
    })
}

/// Whether the readers start again a little before `wanted_us` rather than
/// read on to it: the frame wanted lies behind `held_us`, the one in hand,
/// by more than a frame, as on a reordered timeline that plays an earlier
/// stretch after a later one; or past `next_us`, the next to be read, by
/// more than seeking costs. A still screen records no frames for a while, so
/// the distance ahead is measured from the next frame, not the one in hand.
pub(super) fn should_reposition(
  wanted_us: u64,
  held_us: u64,
  next_us: Option<u64>,
  frame_us: u64,
) -> bool {
  let behind = wanted_us + frame_us < held_us;
  let far_ahead = next_us.is_some_and(|next| wanted_us > next + SKIP_AHEAD_US);
  behind || far_ahead
}

#[cfg(target_os = "macos")]
mod c_api {
  //! The clock as `video_export_cadence.h` declares it.

  use super::*;

  /// # Safety
  /// `ranges` must hold `count` ranges, or be NULL with `count` zero.
  unsafe fn ranges<'a>(ranges: *const TimelineRange, count: u32) -> &'a [TimelineRange] {
    if ranges.is_null() || count == 0 {
      return &[];
    }
    unsafe { std::slice::from_raw_parts(ranges, count as usize) }
  }

  #[no_mangle]
  pub extern "C" fn screenwide_export_frame_rate(nominal: f64) -> u32 {
    frame_rate(Some(nominal))
  }

  /// # Safety
  /// See [`ranges`].
  #[no_mangle]
  pub unsafe extern "C" fn screenwide_export_duration_us(
    source_duration_us: u64,
    timeline: *const TimelineRange,
    count: u32,
  ) -> u64 {
    duration_us(source_duration_us, unsafe { ranges(timeline, count) })
  }

  /// Writes the source moment shown `output_us` into the export to
  /// `source_us` and answers 1, or answers 0 where nothing shows there.
  ///
  /// # Safety
  /// See [`ranges`]; `source_us` must be writable.
  #[no_mangle]
  pub unsafe extern "C" fn screenwide_export_source_time_us(
    output_us: u64,
    timeline: *const TimelineRange,
    count: u32,
    out: *mut u64,
  ) -> i32 {
    match source_us(output_us, unsafe { ranges(timeline, count) }) {
      Some(source) => {
        unsafe { *out = source };
        1
      }
      None => 0,
    }
  }

  /// [`should_reposition`]; `next_us` is `u64::MAX` where there is no next
  /// frame.
  #[no_mangle]
  pub extern "C" fn screenwide_export_should_reposition(
    wanted_us: u64,
    held_us: u64,
    next_us: u64,
    frame_us: u64,
  ) -> i32 {
    let next = (next_us != u64::MAX).then_some(next_us);
    i32::from(should_reposition(wanted_us, held_us, next, frame_us))
  }

  #[no_mangle]
  pub extern "C" fn screenwide_export_rewind_preroll_us() -> u64 {
    REWIND_PREROLL_US
  }
}
