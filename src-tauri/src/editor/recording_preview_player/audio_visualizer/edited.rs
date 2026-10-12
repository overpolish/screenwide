// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The audio-only preview's bars follow the edited recording rather than the
//! source: what was cut is never drawn, and the bars run on through a cut as
//! the sound does instead of jumping ahead over what was taken out.

use screenwide_timeline::{output_at_us, TimelineRange};

/// How long the edited timeline lasts, in microseconds: the kept ranges laid
/// end to end at their own rates. `None` where nothing was cut.
fn edited_length_us(ranges: &[TimelineRange]) -> Option<f64> {
  let last = ranges.last()?;
  let length = last.output_start_us as f64
    + last.source_end_us.saturating_sub(last.source_start_us) as f64 / last.playback_rate;
  (length > 0.0).then_some(length)
}

/// Where `output_us` on the edited timeline plays from in the source.
fn source_at_us(ranges: &[TimelineRange], output_us: f64) -> f64 {
  for range in ranges {
    let length = range.source_end_us.saturating_sub(range.source_start_us) as f64;
    let output_end = range.output_start_us as f64 + length / range.playback_rate;
    if output_us < output_end {
      let into = (output_us - range.output_start_us as f64).max(0.0);
      return range.source_start_us as f64 + (into * range.playback_rate).min(length);
    }
  }
  ranges
    .last()
    .map_or(0.0, |range| range.source_end_us as f64)
}

/// `waveform`, an envelope spread evenly over a source `duration_ms` long, as
/// the edited timeline hears it, each point read from where that moment of
/// the edited recording plays in the source. The ribbon draws one bar per few
/// points and scrolls through all of them over the recording, so the point
/// count shrinks with the edit to keep the bars moving at the same pace.
pub(super) fn edited_waveform(
  waveform: &[f32],
  ranges: &[TimelineRange],
  duration_ms: u64,
) -> Vec<f32> {
  let Some(length) = edited_length_us(ranges) else {
    return waveform.to_vec();
  };
  if duration_ms == 0 || waveform.is_empty() {
    return waveform.to_vec();
  }
  let source_us = duration_ms as f64 * 1_000.0;
  let points = ((waveform.len() as f64 * length / source_us).ceil() as usize).max(1);
  (0..points)
    .map(|point| {
      let output_us = (point as f64 + 0.5) / points as f64 * length;
      let share = source_at_us(ranges, output_us) / source_us;
      let index = ((share * waveform.len() as f64) as usize).min(waveform.len() - 1);
      waveform[index]
    })
    .collect()
}

/// Where `source_ms` stands on the edited timeline, from 0 to 1. A moment
/// that was cut stands where the cut joins what is kept either side.
pub(crate) fn edited_ratio(ranges: &[TimelineRange], source_ms: f64, duration_ms: u64) -> f64 {
  match edited_length_us(ranges) {
    Some(length) => (output_at_us(ranges, source_ms * 1_000.0) / length).clamp(0.0, 1.0),
    None if duration_ms > 0 => (source_ms / duration_ms as f64).clamp(0.0, 1.0),
    None => 0.0,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Keeps 0-1 s and 3-4 s of a 4 s recording: 2 s of output.
  fn ranges() -> [TimelineRange; 2] {
    [
      TimelineRange {
        output_start_us: 0,
        playback_rate: 1.0,
        source_end_us: 1_000_000,
        source_start_us: 0,
      },
      TimelineRange {
        output_start_us: 1_000_000,
        playback_rate: 1.0,
        source_end_us: 4_000_000,
        source_start_us: 3_000_000,
      },
    ]
  }

  #[test]
  fn what_was_cut_is_never_drawn() {
    // One point a second: the 1-3 s points are loud, and they were cut.
    let waveform = [0.1, 0.9, 0.9, 0.2];
    assert_eq!(edited_waveform(&waveform, &ranges(), 4_000), [0.1, 0.2]);
  }

  #[test]
  fn the_playhead_runs_on_through_a_cut() {
    assert_eq!(edited_ratio(&ranges(), 500.0, 4_000), 0.25);
    // Inside the cut it stands at the join, and after it carries straight on.
    assert_eq!(edited_ratio(&ranges(), 2_000.0, 4_000), 0.5);
    assert_eq!(edited_ratio(&ranges(), 3_500.0, 4_000), 0.75);
  }

  #[test]
  fn an_uncut_recording_draws_as_it_is() {
    assert_eq!(edited_waveform(&[0.3, 0.6], &[], 4_000), [0.3, 0.6]);
    assert_eq!(edited_ratio(&[], 1_000.0, 4_000), 0.25);
  }
}
