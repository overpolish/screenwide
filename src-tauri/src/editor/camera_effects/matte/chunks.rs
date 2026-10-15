// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the camera's matte is made: over the stretches of camera the
//! background ranges cover and nothing more, in chunks of at most ten
//! seconds so several are made side by side and each shows as soon as it is
//! made, nearest the playhead first. The ranges are timed on the edited
//! video and the chunks on the camera, so each is carried onto the other
//! through the edit's timeline.
//!
//! Every span here is `[start_ms, end_ms)` in milliseconds.

use screenwide_timeline::TimelineRange;
use screenwide_timeline_edit::CameraBackgroundRange;

pub(super) type Span = [u64; 2];

/// The longest chunk made, in milliseconds. A longer stretch is split into
/// chunks no longer than this, of even length.
const LONGEST_MS: u64 = 10_000;
/// How far past each end of a range the matte is made, so the frame at the
/// very edge of a range has one wherever the edge falls between frames.
const MARGIN_MS: u64 = 100;

/// Where a stretch of the timeline `timeline` lays out lands in the
/// recording, or the other way round: each piece of `[from_us, to_us)` that
/// one range of `timeline` plays, as `[start, end)` on the other side, in
/// microseconds. With no ranges the recording plays whole and unchanged.
fn carried(timeline: &[TimelineRange], [from_us, to_us]: Span, to_source: bool) -> Vec<Span> {
  if timeline.is_empty() {
    return vec![[from_us, to_us]];
  }
  timeline
    .iter()
    .filter_map(|range| {
      let output_end_us = range.output_start_us
        + ((range.source_end_us - range.source_start_us) as f64 / range.playback_rate).round()
          as u64;
      let ((start, end), (other_start, scale)) = if to_source {
        (
          (range.output_start_us, output_end_us),
          (range.source_start_us, range.playback_rate),
        )
      } else {
        (
          (range.source_start_us, range.source_end_us),
          (range.output_start_us, 1.0 / range.playback_rate),
        )
      };
      let (from, to) = (from_us.max(start), to_us.min(end));
      (from < to).then(|| {
        let at = |us: u64| other_start + ((us - start) as f64 * scale).round() as u64;
        [at(from), at(to)]
      })
    })
    .collect()
}

/// `spans` in order, those that meet or overlap joined.
fn joined(mut spans: Vec<Span>) -> Vec<Span> {
  spans.sort_unstable();
  let mut joined: Vec<Span> = Vec::with_capacity(spans.len());
  for [start, end] in spans {
    match joined.last_mut() {
      Some(last) if start <= last[1] => last[1] = last[1].max(end),
      _ => joined.push([start, end]),
    }
  }
  joined
}

/// The camera `camera_ms` long that `ranges` cover, on the timeline
/// `timeline` lays out.
pub(super) fn needed(
  ranges: &[CameraBackgroundRange],
  timeline: &[TimelineRange],
  camera_ms: u64,
) -> Vec<Span> {
  joined(
    ranges
      .iter()
      .flat_map(|range| {
        carried(
          timeline,
          [range.start_ms * 1_000, range.end_ms * 1_000],
          true,
        )
      })
      .map(|[start_us, end_us]| {
        [
          (start_us / 1_000).saturating_sub(MARGIN_MS),
          (end_us.div_ceil(1_000) + MARGIN_MS).min(camera_ms),
        ]
      })
      .filter(|[start, end]| start < end)
      .collect(),
  )
}

/// What of `needed` none of `made` covers.
pub(super) fn missing(needed: &[Span], made: &[Span]) -> Vec<Span> {
  let made = joined(made.to_vec());
  needed
    .iter()
    .flat_map(|&[start, end]| {
      let mut left = Vec::new();
      let mut from = start;
      for &[made_start, made_end] in &made {
        if made_end <= from || made_start >= end {
          continue;
        }
        if made_start > from {
          left.push([from, made_start]);
        }
        from = from.max(made_end);
      }
      if from < end {
        left.push([from, end]);
      }
      left
    })
    .collect()
}

/// `missing` cut into the chunks it is made in: each stretch in as few
/// chunks of even length as keep them no longer than [`LONGEST_MS`].
pub(super) fn chunks(missing: &[Span]) -> Vec<Span> {
  missing
    .iter()
    .flat_map(|&[start, end]| {
      let count = (end - start).div_ceil(LONGEST_MS).max(1);
      (0..count).map(move |index| {
        [
          start + (end - start) * index / count,
          start + (end - start) * (index + 1) / count,
        ]
      })
    })
    .collect()
}

/// `chunks` in the order they are made: from the one the playhead is in at
/// `playhead_ms` onward, then back from it, so what is about to play is
/// ready first.
pub(super) fn nearest_first(mut chunks: Vec<Span>, playhead_ms: u64) -> Vec<Span> {
  chunks.sort_unstable();
  let (before, after): (Vec<Span>, Vec<Span>) =
    chunks.into_iter().partition(|[_, end]| *end <= playhead_ms);
  after.into_iter().chain(before.into_iter().rev()).collect()
}

/// Where `spans` of the camera play on the timeline `timeline` lays out, in
/// order and joined where they meet.
pub(super) fn timeline_spans(spans: &[Span], timeline: &[TimelineRange]) -> Vec<Span> {
  joined(
    spans
      .iter()
      .flat_map(|&[start, end]| carried(timeline, [start * 1_000, end * 1_000], false))
      .map(|[start_us, end_us]| [start_us / 1_000, end_us.div_ceil(1_000)])
      .collect(),
  )
}

/// Whether `made` covers the whole camera, `camera_ms` long.
pub(super) fn whole(made: &[Span], camera_ms: u64) -> bool {
  missing(&[[0, camera_ms]], made).is_empty()
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_timeline_edit::CameraRangeBackground;

  fn range(start_ms: u64, end_ms: u64) -> CameraBackgroundRange {
    CameraBackgroundRange {
      id: "range".to_owned(),
      start_ms,
      end_ms,
      background: CameraRangeBackground::Blurred,
    }
  }

  /// 0-20 s plays, 20-40 s is cut away, and 40-60 s plays at double speed.
  fn cut_and_fast() -> Vec<TimelineRange> {
    vec![
      TimelineRange {
        output_start_us: 0,
        source_start_us: 0,
        source_end_us: 20_000_000,
        playback_rate: 1.0,
      },
      TimelineRange {
        output_start_us: 20_000_000,
        source_start_us: 40_000_000,
        source_end_us: 60_000_000,
        playback_rate: 2.0,
      },
    ]
  }

  #[test]
  fn a_range_needs_its_own_stretch_of_camera_and_a_frame_either_side() {
    assert_eq!(
      needed(&[range(12_000, 22_000)], &[], 180_000),
      [[11_900, 22_100]]
    );
  }

  #[test]
  fn a_range_across_a_cut_needs_the_footage_on_both_sides() {
    // 15-25 s of the timeline plays 15-20 s and 40-50 s of the camera.
    assert_eq!(
      needed(&[range(15_000, 25_000)], &cut_and_fast(), 60_000),
      [[14_900, 20_100], [39_900, 50_100]]
    );
  }

  #[test]
  fn only_what_is_not_made_is_made_again() {
    let left = missing(&[[0, 30_000]], &[[5_000, 10_000], [20_000, 40_000]]);
    assert_eq!(left, [[0, 5_000], [10_000, 20_000]]);
  }

  #[test]
  fn a_long_stretch_is_made_in_even_chunks_of_ten_seconds_at_most() {
    assert_eq!(
      chunks(&[[0, 25_000]]),
      [[0, 8_333], [8_333, 16_666], [16_666, 25_000]]
    );
    assert_eq!(chunks(&[[2_000, 12_000]]), [[2_000, 12_000]]);
  }

  #[test]
  fn chunks_are_made_from_the_playhead_onward_then_back() {
    let chunks = vec![
      [0, 5_000],
      [5_000, 10_000],
      [10_000, 15_000],
      [20_000, 25_000],
    ];
    assert_eq!(
      nearest_first(chunks, 12_000),
      [
        [10_000, 15_000],
        [20_000, 25_000],
        [5_000, 10_000],
        [0, 5_000]
      ]
    );
  }

  #[test]
  fn missing_camera_shows_where_it_plays_on_the_timeline() {
    // 10-20 s plays at once; 40-50 s plays at double speed from 20 s.
    let spans = timeline_spans(&[[10_000, 20_000], [40_000, 50_000]], &cut_and_fast());
    assert_eq!(spans, [[10_000, 25_000]]);
  }

  #[test]
  fn the_whole_camera_is_made_once_nothing_is_left_uncovered() {
    assert!(whole(&[[0, 10_000], [10_000, 25_000]], 25_000));
    assert!(!whole(&[[0, 10_000], [12_000, 25_000]], 25_000));
  }
}
