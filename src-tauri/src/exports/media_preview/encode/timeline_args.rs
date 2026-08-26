// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::exports::timeline_edit::{TimelinePlan, TimelineRange};

const CUT_FADE_US: u64 = 3_000;

fn seconds(microseconds: u64) -> String {
  format!("{:.6}", microseconds as f64 / 1_000_000.0)
}

fn cut_fades(range: TimelineRange, index: usize, count: usize) -> String {
  let mut fades = String::new();
  if index > 0 {
    fades.push_str(&format!(",afade=t=in:st=0:d={}", seconds(CUT_FADE_US)));
  }
  if index + 1 < count {
    let duration_us = range.source_end_us.saturating_sub(range.source_start_us);
    fades.push_str(&format!(
      ",afade=t=out:st={}:d={}",
      seconds(duration_us.saturating_sub(CUT_FADE_US)),
      seconds(CUT_FADE_US)
    ));
  }
  fades
}

fn video_encoding_args(video: VideoExportOptions) -> Vec<OsString> {
  let crf = export_crf(video.compression, true).unwrap_or(20);
  [
    "-c:v",
    "libx264",
    "-preset",
    "medium",
    "-crf",
    &crf.to_string(),
    "-pix_fmt",
    "yuv420p",
    "-profile:v",
    "high",
  ]
  .map(OsString::from)
  .to_vec()
}

fn append_video_filters(
  filters: &mut Vec<String>,
  args: &mut Vec<OsString>,
  timeline: &TimelinePlan,
  input: usize,
  video: VideoExportOptions,
) {
  let ranges = timeline.ranges();
  if ranges.len() > 1 {
    let outputs: String = (0..ranges.len())
      .map(|index| format!("[vs{index}]"))
      .collect();
    filters.push(format!("[{input}:v:0]split={}{outputs}", ranges.len()));
  }
  let mut parts = String::new();
  for (index, range) in ranges.iter().enumerate() {
    let source = if ranges.len() > 1 {
      format!("[vs{index}]")
    } else {
      format!("[{input}:v:0]")
    };
    filters.push(format!(
      "{source}trim=start={}:end={},setpts=PTS-STARTPTS[v{index}]",
      seconds(range.source_start_us),
      seconds(range.source_end_us)
    ));
    parts.push_str(&format!("[v{index}]"));
  }
  let scale = resolution_filter(video.source_scale_percent, video.resolution_scale_percent)
    .map_or_else(String::new, |filter| format!(",{filter}"));
  filters.push(format!(
    "{parts}concat=n={}:v=1:a=0{scale}[vout]",
    ranges.len()
  ));
  args.extend([OsString::from("-map"), OsString::from("[vout]")]);
  args.extend(video_encoding_args(video));
}

fn append_audio_filters(
  filters: &mut Vec<String>,
  args: &mut Vec<OsString>,
  timeline: &TimelinePlan,
  audio_input: usize,
  selection: &TrackSelection,
  layout: AudioLayout,
) {
  let ranges = timeline.ranges();
  let mut outputs = Vec::new();
  for (track, stream) in selection.stream_indices().iter().enumerate() {
    if ranges.len() > 1 {
      let splits: String = (0..ranges.len())
        .map(|index| format!("[as{track}_{index}]"))
        .collect();
      filters.push(format!(
        "[{audio_input}:a:{stream}]asplit={}{splits}",
        ranges.len()
      ));
    }
    let mut parts = String::new();
    for (index, range) in ranges.iter().enumerate() {
      let source = if ranges.len() > 1 {
        format!("[as{track}_{index}]")
      } else {
        format!("[{audio_input}:a:{stream}]")
      };
      let fades = cut_fades(*range, index, ranges.len());
      filters.push(format!(
        "{source}atrim=start={}:end={},asetpts=PTS-STARTPTS{fades}[a{track}_{index}]",
        seconds(range.source_start_us),
        seconds(range.source_end_us)
      ));
      parts.push_str(&format!("[a{track}_{index}]"));
    }
    let volume = selection.volume_decibels(*stream);
    let volume = if volume == 0 {
      String::new()
    } else {
      format!(",volume={volume}dB")
    };
    filters.push(format!(
      "{parts}concat=n={}:v=0:a=1{volume}[at{track}]",
      ranges.len()
    ));
    outputs.push(format!("[at{track}]"));
  }
  if layout == AudioLayout::Mixdown && outputs.len() > 1 {
    filters.push(format!(
      "{}amix=inputs={}:normalize=0[aout]",
      outputs.concat(),
      outputs.len()
    ));
    args.extend([OsString::from("-map"), OsString::from("[aout]")]);
  } else {
    for output in outputs {
      args.extend([OsString::from("-map"), OsString::from(output)]);
    }
  }
  if !selection.stream_indices().is_empty() {
    args.extend(["-c:a", "aac", "-b:a", "192k"].map(OsString::from));
  }
}

fn filter_args(
  timeline: &TimelinePlan,
  video_input: Option<usize>,
  audio_input: usize,
  selection: &TrackSelection,
  layout: AudioLayout,
  video: VideoExportOptions,
) -> Vec<OsString> {
  let mut filters = Vec::new();
  let mut args = Vec::new();
  if let Some(input) = video_input {
    append_video_filters(&mut filters, &mut args, timeline, input, video);
  }
  append_audio_filters(
    &mut filters,
    &mut args,
    timeline,
    audio_input,
    selection,
    layout,
  );
  if filters.is_empty() {
    return vec![OsString::from("-an")];
  }
  let mut result = vec![
    OsString::from("-filter_complex"),
    OsString::from(filters.join(";")),
  ];
  result.extend(args);
  result
}

fn base_args(source: &Path) -> Vec<OsString> {
  let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]
    .map(OsString::from)
    .into();
  args.push(source.into());
  args
}

fn finish(mut args: Vec<OsString>, destination: &Path) -> Vec<OsString> {
  args.extend(EXPORT_MP4_OUTPUT.map(OsString::from));
  args.push(destination.into());
  args
}

pub(in crate::exports) fn timeline_audio_mapping_args(
  timeline: &TimelinePlan,
  audio_input: usize,
  selection: &TrackSelection,
  layout: AudioLayout,
) -> Vec<OsString> {
  filter_args(
    timeline,
    None,
    audio_input,
    selection,
    layout,
    VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  )
}

pub(in crate::exports::media_preview) fn timeline_selected_export_args(
  source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  video: VideoExportOptions,
  timeline: &TimelinePlan,
) -> Vec<OsString> {
  let mut args = base_args(source);
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(filter_args(timeline, Some(0), 0, selection, layout, video));
  finish(args, destination)
}

pub(in crate::exports::media_preview) fn timeline_camera_export_args(
  audio_source: &Path,
  camera_source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  video: VideoExportOptions,
  timeline: &TimelinePlan,
) -> Vec<OsString> {
  let mut args = base_args(audio_source);
  args.push(OsString::from("-i"));
  args.push(camera_source.into());
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(filter_args(timeline, Some(1), 0, selection, layout, video));
  finish(args, destination)
}

pub(in crate::exports::media_preview) fn timeline_audio_export_args(
  source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  timeline: &TimelinePlan,
) -> Vec<OsString> {
  let mut args = base_args(source);
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(timeline_audio_mapping_args(timeline, 0, selection, layout));
  finish(args, destination)
}

#[cfg(test)]
mod tests {
  use super::{cut_fades, TimelineRange};

  const RANGE: TimelineRange = TimelineRange {
    output_start_us: 0,
    source_end_us: 250_000,
    source_start_us: 0,
  };

  #[test]
  fn smooths_both_sides_of_a_cut_without_changing_range_duration() {
    assert_eq!(
      cut_fades(RANGE, 0, 2),
      ",afade=t=out:st=0.247000:d=0.003000"
    );
    assert_eq!(cut_fades(RANGE, 1, 2), ",afade=t=in:st=0:d=0.003000");
    assert!(cut_fades(RANGE, 0, 1).is_empty());
  }
}
