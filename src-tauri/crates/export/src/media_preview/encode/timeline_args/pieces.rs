// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each range of the timeline as a piece of FFmpeg's filter graph, from one
//! branch of the source split for it: a stretch of the recording trimmed
//! out and played at its speed, or a gap, which has nothing recorded.

use screenwide_timeline::TimelineRange;

const CUT_FADE_US: u64 = 3_000;

fn seconds(microseconds: u64) -> String {
  format!("{:.6}", microseconds as f64 / 1_000_000.0)
}

pub(super) fn atempo(rate: f64) -> String {
  let mut remaining = rate;
  let mut filters = Vec::new();
  while remaining > 2.0 {
    filters.push("atempo=2.0".to_owned());
    remaining /= 2.0;
  }
  while remaining < 0.5 {
    filters.push("atempo=0.5".to_owned());
    remaining /= 0.5;
  }
  if (remaining - 1.0).abs() > f64::EPSILON {
    filters.push(format!("atempo={remaining:.6}"));
  }
  filters.join(",")
}

pub(super) fn cut_fades(range: TimelineRange, index: usize, count: usize) -> String {
  let mut fades = String::new();
  if index > 0 {
    fades.push_str(&format!(",afade=t=in:st=0:d={}", seconds(CUT_FADE_US)));
  }
  if index + 1 < count {
    let duration_us = ((range.source_end_us.saturating_sub(range.source_start_us)) as f64
      / range.playback_rate)
      .round() as u64;
    fades.push_str(&format!(
      ",afade=t=out:st={}:d={}",
      seconds(duration_us.saturating_sub(CUT_FADE_US)),
      seconds(CUT_FADE_US)
    ));
  }
  fades
}

/// The picture of `range` on a recording `source_duration_us` long. A gap
/// has nothing recorded: one frame is taken for its size and rate, padded
/// with black for the gap's length and then dropped.
pub(super) fn video_piece(range: &TimelineRange, source_duration_us: u64) -> String {
  if range.is_gap(source_duration_us) {
    return format!(
      "trim=end_frame=1,setpts=PTS-STARTPTS,tpad=stop_mode=add:stop_duration={}:color=black,trim=start_frame=1,setpts=PTS-STARTPTS",
      seconds(range.output_end_us() - range.output_start_us)
    );
  }
  format!(
    "trim=start={}:end={},setpts=(PTS-STARTPTS)/{}",
    seconds(range.source_start_us),
    seconds(range.source_end_us),
    range.playback_rate
  )
}

/// The sound of `range`, the `index`th of `count`, on a recording
/// `source_duration_us` long, faded in and out where it meets another. A gap
/// sounds nothing: one muted sample, padded with silence for its length.
pub(super) fn audio_piece(
  range: &TimelineRange,
  (index, count): (usize, usize),
  source_duration_us: u64,
) -> String {
  if range.is_gap(source_duration_us) {
    return format!(
      "atrim=end_sample=1,asetpts=PTS-STARTPTS,volume=0,apad=whole_dur={}",
      seconds(range.output_end_us() - range.output_start_us)
    );
  }
  let tempo = atempo(range.playback_rate);
  let tempo = if tempo.is_empty() {
    String::new()
  } else {
    format!(",{tempo}")
  };
  format!(
    "atrim=start={}:end={},asetpts=PTS-STARTPTS{tempo}{}",
    seconds(range.source_start_us),
    seconds(range.source_end_us),
    cut_fades(*range, index, count)
  )
}
