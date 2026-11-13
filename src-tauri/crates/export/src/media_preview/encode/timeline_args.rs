// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod commands;
#[cfg(test)]
mod tests;

pub(crate) use commands::{
  timeline_audio_export_args, timeline_audio_mapping_args, timeline_camera_export_args,
  timeline_selected_export_args,
};

use super::*;
use screenwide_timeline::TimelineRange;
use screenwide_timeline_edit::TimelinePlan;

const CUT_FADE_US: u64 = 3_000;
/// Where the Mixdown's peaks are held once songs and sound effects are
/// summed with the recorded tracks, which could otherwise clip.
const MIX_LIMITER: &str = "alimiter=limit=-1dB:attack=5:release=50:level=0:latency=1";

/// The timeline's songs and sound effects mixed into `[sound]`, if any are
/// heard. Those that make way listen to the microphone as exported, which
/// `outputs` hold one track each of, in `selection`'s order: it is split, so
/// it is still exported as it was. The other tracks, the system audio, make
/// room for the sound effects, and are read from the labels the sound track
/// hands back.
fn sound_track_filter(
  timeline: &TimelinePlan,
  selection: &TrackSelection,
  (filters, outputs): (&mut Vec<String>, &mut [String]),
) -> Option<String> {
  let sound_track = timeline.sound_track();
  if sound_track.clips.is_empty() {
    return None;
  }
  let timeline_ms = timeline.duration_ms();
  let pieces = sound_track.pieces(timeline_ms);
  let voice_track = sound_track.voice.and_then(|stream| {
    selection
      .stream_indices()
      .iter()
      .position(|each| *each == stream)
  });
  let voice = voice_track
    .filter(|_| screenwide_sound_track::graph::makes_way(&pieces))
    .map(|track| {
      filters.push(format!(
        "{}asplit=2[at{track}out][soundvoice]",
        outputs[track]
      ));
      outputs[track] = format!("[at{track}out]");
      "[soundvoice]"
    });
  let others: Vec<usize> = (0..outputs.len())
    .filter(|track| Some(*track) != voice_track)
    .collect();
  let beside: Vec<String> = others.iter().map(|track| outputs[*track].clone()).collect();
  let mix = screenwide_sound_track::graph::filter(
    &pieces,
    (0, timeline_ms),
    ("sound", ""),
    voice,
    &beside,
  )?;
  for (track, label) in others.into_iter().zip(mix.beside) {
    outputs[track] = label;
  }
  Some(mix.graph)
}

fn seconds(microseconds: u64) -> String {
  format!("{:.6}", microseconds as f64 / 1_000_000.0)
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

fn cut_fades(range: TimelineRange, index: usize, count: usize) -> String {
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

/// x264 at the export's quality: an export that follows an edit always
/// re-encodes, Original included.
pub(in crate::media_preview) fn video_encoding_args(video: VideoExportOptions) -> Vec<OsString> {
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
      "{source}trim=start={}:end={},setpts=(PTS-STARTPTS)/{}[v{index}]",
      seconds(range.source_start_us),
      seconds(range.source_end_us),
      range.playback_rate
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
  encoding: AudioEncoding,
) {
  let ranges = timeline.ranges();
  let mut outputs = Vec::new();
  let opened = selection.opened_sources(audio_input);
  if !opened.is_empty() {
    filters.push(opened.trim_end_matches(';').to_owned());
  }
  for (track, stream) in selection.stream_indices().iter().enumerate() {
    let label = selection.source(audio_input, *stream);
    if ranges.len() > 1 {
      let splits: String = (0..ranges.len())
        .map(|index| format!("[as{track}_{index}]"))
        .collect();
      filters.push(format!("{label}asplit={}{splits}", ranges.len()));
    }
    let mut parts = String::new();
    for (index, range) in ranges.iter().enumerate() {
      let source = if ranges.len() > 1 {
        format!("[as{track}_{index}]")
      } else {
        label.clone()
      };
      let fades = cut_fades(*range, index, ranges.len());
      let tempo = atempo(range.playback_rate);
      let tempo = if tempo.is_empty() {
        String::new()
      } else {
        format!(",{tempo}")
      };
      filters.push(format!(
        "{source}atrim=start={}:end={},asetpts=PTS-STARTPTS{tempo}{fades}[a{track}_{index}]",
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
  let sound = sound_track_filter(timeline, selection, (filters, &mut outputs));
  if let Some(graph) = &sound {
    filters.push(graph.clone());
  }
  let sound_output = sound.as_ref().map(|_| "[sound]".to_owned());
  match layout {
    AudioLayout::Mixdown if outputs.len() + usize::from(sound.is_some()) > 1 => {
      let inputs: String = outputs
        .iter()
        .chain(&sound_output)
        .map(String::as_str)
        .collect();
      let count = outputs.len() + usize::from(sound.is_some());
      let limiter = if sound.is_some() {
        format!(",{MIX_LIMITER}")
      } else {
        String::new()
      };
      filters.push(format!(
        "{inputs}amix=inputs={count}:normalize=0{limiter}[aout]"
      ));
      args.extend([OsString::from("-map"), OsString::from("[aout]")]);
    }
    _ => {
      // Kept apart, the songs and sound effects are one stream of their own
      // after the recorded tracks.
      for output in outputs.into_iter().chain(sound_output) {
        args.extend([OsString::from("-map"), OsString::from(output)]);
      }
    }
  }
  if !selection.stream_indices().is_empty() || sound.is_some() {
    args.extend(encoding.codec_args(false).into_iter().map(OsString::from));
  }
}

fn filter_args(
  timeline: &TimelinePlan,
  video_input: Option<usize>,
  audio_input: usize,
  selection: &TrackSelection,
  layout: AudioLayout,
  encoding: AudioEncoding,
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
    encoding,
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
