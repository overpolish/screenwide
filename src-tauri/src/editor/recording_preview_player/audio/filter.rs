// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cpal::StreamConfig;

use super::super::sound_track::PreviewSoundTrack;
use super::super::{PlayerSources, RecordingPreviewPlaybackRange};
use super::Channel;

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

fn cut_fades(range: RecordingPreviewPlaybackRange, index: usize, count: usize) -> String {
  let mut fades = String::new();
  if index > 0 {
    fades.push_str(&format!(",afade=t=in:st=0:d={}", seconds(CUT_FADE_MS)));
  }
  if index + 1 < count {
    let fade_start_ms = ((range.duration_ms() as f64 / range.playback_rate).ceil() as u64)
      .saturating_sub(CUT_FADE_MS);
    fades.push_str(&format!(
      ",afade=t=out:st={}:d={}",
      seconds(fade_start_ms),
      seconds(CUT_FADE_MS)
    ));
  }
  fades
}

/// The FFmpeg command that decodes `channels` over `ranges`, then `sound`,
/// the songs and sound effects mixed from where playback starts, as one
/// channel more. Those that make way listen to the microphone as recorded.
pub(super) fn args(
  sources: &PlayerSources,
  channels: &[Channel],
  sound: Option<&PreviewSoundTrack>,
  ranges: &[RecordingPreviewPlaybackRange],
  config: &StreamConfig,
) -> Vec<String> {
  // The microphone as recorded, which the clips that make way listen to,
  // split from its channel only where the clips are heard.
  let voice_track = sound.and_then(PreviewSoundTrack::voice).and_then(|stream| {
    channels.iter().position(|channel| {
      channel.stream_index == stream && channel.processed.is_none() && channel.leveling.is_none()
    })
  });
  // The tracks other than the microphone, the system audio, make room for
  // the sound effects: each is handed to the songs and sound effects by its
  // label, and heard from the label handed back.
  let voice_stream = sound.and_then(PreviewSoundTrack::voice_stream);
  let beside: Vec<usize> = channels
    .iter()
    .enumerate()
    .filter(|(_, channel)| Some(channel.stream_index) != voice_stream)
    .map(|(position, _)| position)
    .collect();
  let beside_labels: Vec<String> = beside
    .iter()
    .map(|position| format!("[track{position}]"))
    .collect();
  let sound =
    sound.and_then(|sound| sound.filter(voice_track.map(|_| "[soundvoice]"), &beside_labels));
  let voice_track = voice_track.filter(|_| sound.is_some());
  let heard_label = |position: usize| {
    sound
      .as_ref()
      .and_then(|mix| {
        beside
          .iter()
          .position(|each| *each == position)
          .map(|index| mix.beside[index].clone())
      })
      .unwrap_or_else(|| format!("[track{position}]"))
  };
  // Every range is trimmed from one decode of the source, which therefore
  // starts at the range earliest in it: on a reordered timeline that need
  // not be the first one played.
  let first_start_ms = ranges
    .iter()
    .map(|range| range.source_start_ms)
    .min()
    .unwrap_or(0);
  let start = seconds(first_start_ms);
  let mut args = vec![
    "-hide_banner".to_owned(),
    "-loglevel".to_owned(),
    "error".to_owned(),
    "-nostdin".to_owned(),
    "-ss".to_owned(),
    start.clone(),
    "-i".to_owned(),
    sources.screen_path.to_string_lossy().into_owned(),
  ];
  // A processed track is a file of its own, sought to the same place: each
  // is an input after the recording, in channel order.
  let mut next_input = 1;
  let mut filter = String::new();
  let labels: Vec<String> = channels
    .iter()
    .enumerate()
    .map(|(position, channel)| {
      let label = match &channel.processed {
        None => format!("[0:a:{}]", channel.stream_index),
        Some((_, path)) => {
          args.extend([
            "-ss".to_owned(),
            start.clone(),
            "-i".to_owned(),
            path.to_string_lossy().into_owned(),
          ]);
          next_input += 1;
          format!("[{}:a:0]", next_input - 1)
        }
      };
      match &channel.leveling {
        None => label,
        Some(leveling) => {
          filter.push_str(&format!("{label}{}[level{position}];", leveling.filters()));
          format!("[level{position}]")
        }
      }
    })
    .collect();
  let retained = seconds(
    ranges
      .iter()
      .map(|range| (range.duration_ms() as f64 / range.playback_rate).ceil() as u64)
      .sum(),
  );
  let mut merged_inputs = String::new();
  for (track_position, label) in labels.iter().enumerate() {
    if ranges.len() > 1 {
      let split_inputs = (0..ranges.len())
        .map(|range_index| format!("[track{track_position}source{range_index}]"))
        .collect::<String>();
      filter.push_str(&format!("{label}asplit={}{split_inputs};", ranges.len()));
    }
    let mut range_inputs = String::new();
    for (range_index, range) in ranges.iter().enumerate() {
      let relative_start = range.source_start_ms.saturating_sub(first_start_ms);
      let relative_end = range.source_end_ms.saturating_sub(first_start_ms);
      let source = if ranges.len() == 1 {
        label.clone()
      } else {
        format!("[track{track_position}source{range_index}]")
      };
      let fades = cut_fades(*range, range_index, ranges.len());
      let tempo = atempo(range.playback_rate);
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
    let heard = if voice_track == Some(track_position) {
      format!("[track{track_position}heard]")
    } else {
      format!("[track{track_position}]")
    };
    filter.push_str(&format!(
      "{range_inputs}concat=n={}:v=0:a=1,aresample={},aformat=sample_fmts=flt:channel_layouts=mono,apad=whole_dur={retained}{heard};",
      ranges.len(), config.sample_rate,
    ));
    if voice_track == Some(track_position) {
      filter.push_str(&format!(
        "{heard}asplit=2[track{track_position}][soundvoice];"
      ));
    }
    merged_inputs.push_str(&heard_label(track_position));
  }
  // The songs and sound effects, already mixed and timed from where playback
  // starts on the timeline, are one more channel after the tracks.
  let mut count = labels.len();
  if let Some(sound) = &sound {
    filter.push_str(&format!(
      "{};[soundmix]aresample={},aformat=sample_fmts=flt:channel_layouts=mono,apad=whole_dur={retained}[track{count}];",
      sound.graph, config.sample_rate,
    ));
    merged_inputs.push_str(&format!("[track{count}]"));
    count += 1;
  }
  let output = if count == 1 {
    filter.pop();
    "[track0]".to_owned()
  } else {
    filter.push_str(&format!("{merged_inputs}amerge=inputs={count}[tracks]"));
    "[tracks]".to_owned()
  };
  args.extend([
    "-filter_complex".to_owned(),
    filter,
    "-map".to_owned(),
    output,
    "-vn".to_owned(),
    "-ac".to_owned(),
    count.to_string(),
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
