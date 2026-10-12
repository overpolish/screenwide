// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a frame falls on the timeline, and where it falls in a clip timed
//! there: what annotations and scenes alike are placed by.

use screenwide_timeline::{output_at_us, output_duration_us, TimelineRange};

/// Where a drawn frame falls: `source_ms` into the recording, where the
/// picture and whatever follows it are read, and `timeline_ms` into the
/// edited video, where every clip is timed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTime {
  pub source_ms: u64,
  pub timeline_ms: u64,
}

impl FrameTime {
  /// The frame `source_ms` into the recording, on the timeline `ranges` lay
  /// out. With no ranges the recording plays whole.
  pub fn on(ranges: &[TimelineRange], source_ms: u64) -> Self {
    let timeline_ms = if ranges.is_empty() {
      source_ms
    } else {
      (output_at_us(ranges, source_ms as f64 * 1_000.0) / 1_000.0).floor() as u64
    };
    Self {
      source_ms,
      timeline_ms,
    }
  }
}

/// How long the edited video plays, in milliseconds; `None` with no
/// timeline to measure.
pub fn video_ms(ranges: &[TimelineRange]) -> Option<u64> {
  output_duration_us(ranges).map(|us| us.saturating_add(500) / 1_000)
}

/// Whether `[start_ms, end_ms)` on the timeline holds `timeline_ms`. Bounds
/// are half open, so a clip ends exactly where the next one may begin, but
/// one that reaches the end of a video `video_ms` long holds to it: rounding
/// can leave its end a millisecond short of the last frame.
pub fn covers(video_ms: Option<u64>, [start_ms, end_ms]: [u64; 2], timeline_ms: u64) -> bool {
  start_ms <= timeline_ms
    && (timeline_ms < end_ms || reaches_video_ends(video_ms, [start_ms, end_ms]).1)
}

/// How far into `[start_ms, end_ms)` the timeline is at `timeline_ms`, and
/// how long the stretch plays for, in milliseconds; `None` for one that
/// takes no time.
pub fn progress([start_ms, end_ms]: [u64; 2], timeline_ms: u64) -> Option<(f32, f32)> {
  (end_ms > start_ms).then(|| {
    (
      timeline_ms.saturating_sub(start_ms) as f32,
      (end_ms - start_ms) as f32,
    )
  })
}

/// Whether `[start_ms, end_ms)` reaches the start and the end of a video
/// `video_ms` long, to within a millisecond of rounding: there is nothing
/// before it to arrive from, or after it to leave for. With no video to
/// measure it reaches neither.
pub fn reaches_video_ends(video_ms: Option<u64>, [start_ms, end_ms]: [u64; 2]) -> (bool, bool) {
  (
    video_ms.is_some() && start_ms <= 1,
    video_ms.is_some_and(|video_ms| end_ms + 1 >= video_ms),
  )
}
