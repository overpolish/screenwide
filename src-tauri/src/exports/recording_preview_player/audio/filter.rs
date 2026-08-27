// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cpal::StreamConfig;

use super::super::{PlayerSources, RecordingPreviewPlaybackRange};

const CUT_FADE_MS: u64 = 3;

fn seconds(milliseconds: u64) -> String {
  format!("{}.{:03}", milliseconds / 1_000, milliseconds % 1_000)
}

fn atempo(rate: f64) -> String {
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

fn cut_fades(
  range: RecordingPreviewPlaybackRange,
  playback_rate: f64,
  index: usize,
  count: usize,
) -> String {
  let mut fades = String::new();
  if index > 0 {
    fades.push_str(&format!(",afade=t=in:st=0:d={}", seconds(CUT_FADE_MS)));
  }
  if index + 1 < count {
    let effective_rate = range.playback_rate * playback_rate;
    let fade_start_ms =
      ((range.duration_ms() as f64 / effective_rate).ceil() as u64).saturating_sub(CUT_FADE_MS);
    fades.push_str(&format!(
      ",afade=t=out:st={}:d={}",
      seconds(fade_start_ms),
      seconds(CUT_FADE_MS)
    ));
  }
  fades
}

pub(super) fn args(
  sources: &PlayerSources,
  ranges: &[RecordingPreviewPlaybackRange],
  config: &StreamConfig,
  playback_rate: f64,
) -> Vec<String> {
  let stream_indices = sources
    .audio_tracks
    .iter()
    .map(|track| track.stream_index)
    .collect::<Vec<_>>();
  let mut args = vec![
    "-hide_banner".to_owned(),
    "-loglevel".to_owned(),
    "error".to_owned(),
    "-nostdin".to_owned(),
    "-ss".to_owned(),
    seconds(ranges[0].source_start_ms),
    "-i".to_owned(),
    sources.screen_path.to_string_lossy().into_owned(),
  ];
  let first_start_ms = ranges[0].source_start_ms;
  let retained = seconds(
    ranges
      .iter()
      .map(|range| {
        (range.duration_ms() as f64 / (range.playback_rate * playback_rate)).ceil() as u64
      })
      .sum(),
  );
  let mut filter = String::new();
  let mut merged_inputs = String::new();
  for (track_position, stream_index) in stream_indices.iter().enumerate() {
    if ranges.len() > 1 {
      let split_inputs = (0..ranges.len())
        .map(|range_index| format!("[track{track_position}source{range_index}]"))
        .collect::<String>();
      filter.push_str(&format!(
        "[0:a:{stream_index}]asplit={}{split_inputs};",
        ranges.len()
      ));
    }
    let mut range_inputs = String::new();
    for (range_index, range) in ranges.iter().enumerate() {
      let relative_start = range.source_start_ms.saturating_sub(first_start_ms);
      let relative_end = range.source_end_ms.saturating_sub(first_start_ms);
      let source = if ranges.len() == 1 {
        format!("[0:a:{stream_index}]")
      } else {
        format!("[track{track_position}source{range_index}]")
      };
      let fades = cut_fades(*range, playback_rate, range_index, ranges.len());
      let tempo = atempo(range.playback_rate * playback_rate);
      let tempo = if tempo.is_empty() {
        String::new()
      } else {
        format!(",{tempo}")
      };
      filter.push_str(&format!(
        "{source}atrim=start={}:end={},asetpts=PTS-STARTPTS{tempo}{fades}[track{track_position}range{range_index}];",
        seconds(relative_start),
        seconds(relative_end),
      ));
      range_inputs.push_str(&format!("[track{track_position}range{range_index}]"));
    }
    filter.push_str(&format!(
      "{range_inputs}concat=n={}:v=0:a=1,aresample={},aformat=sample_fmts=flt:channel_layouts=mono,apad=whole_dur={retained}[track{track_position}];",
      ranges.len(), config.sample_rate,
    ));
    merged_inputs.push_str(&format!("[track{track_position}]"));
  }
  let output = if stream_indices.len() == 1 {
    filter.pop();
    "[track0]".to_owned()
  } else {
    filter.push_str(&format!(
      "{merged_inputs}amerge=inputs={}[tracks]",
      stream_indices.len()
    ));
    "[tracks]".to_owned()
  };
  args.extend([
    "-filter_complex".to_owned(),
    filter,
    "-map".to_owned(),
    output,
    "-vn".to_owned(),
    "-ac".to_owned(),
    stream_indices.len().to_string(),
    "-ar".to_owned(),
    config.sample_rate.to_string(),
    "-t".to_owned(),
    retained,
    "-f".to_owned(),
    "f32le".to_owned(),
    "pipe:1".to_owned(),
  ]);
  args
}
