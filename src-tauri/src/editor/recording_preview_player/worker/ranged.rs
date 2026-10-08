// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod video;

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc, Mutex, RwLock,
  },
  time::{Duration, Instant},
};

use tauri::ipc::Channel;

use self::video::{PreparedVideo, VideoPlayback, VideoSource};
use super::{audio, platform, send_error, stop_child};
use crate::editor::recording_preview_player::audio::AudioPlayback;
use crate::editor::recording_preview_player::audio_visualizer::present_audio_position;
use crate::editor::recording_preview_player::video::LateFrames;
use crate::editor::recording_preview_player::{
  AudioTrackVolume, PlayerSources, RecordingPreviewPlaybackRange, RecordingPreviewPlayerEvent,
};

pub(super) struct RunContext {
  pub audio_child: Arc<Mutex<Option<Child>>>,
  pub audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  pub cancelled: Arc<AtomicBool>,
  pub event_channel: Channel<RecordingPreviewPlayerEvent>,
  pub playback_end_ms: Option<u64>,
  pub playback_factors: Vec<f64>,
  pub playback_rate: f64,
  pub playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  pub position_ms: Arc<AtomicU64>,
  pub selected_audio: Arc<RwLock<Vec<usize>>>,
  pub sources: PlayerSources,
  pub start_ms: u64,
  pub video_child: Arc<Mutex<Option<Child>>>,
}

fn output_duration_ms(source_duration_ms: u64, playback_rate: f64) -> u64 {
  ((source_duration_ms as f64) / playback_rate).ceil() as u64
}

fn effective_rate(range: RecordingPreviewPlaybackRange, global_rate: f64) -> f64 {
  range.playback_rate * global_rate
}

fn source_elapsed_ms(output_elapsed_ms: u64, playback_rate: f64) -> u64 {
  ((output_elapsed_ms as f64) * playback_rate).round() as u64
}

fn complete_range(position_ms: &AtomicU64, cancelled: &AtomicBool, failed: bool, end_ms: u64) {
  if !failed && !cancelled.load(Ordering::Acquire) {
    position_ms.store(end_ms, Ordering::Release);
  }
}

fn ranges(context: &RunContext) -> Vec<RecordingPreviewPlaybackRange> {
  if context.playback_ranges.is_empty() {
    vec![RecordingPreviewPlaybackRange {
      source_start_ms: context.start_ms,
      source_end_ms: context
        .playback_end_ms
        .unwrap_or(context.sources.duration_ms)
        .min(context.sources.duration_ms)
        .max(context.start_ms),
      playback_rate: 1.0,
    }]
  } else {
    context.playback_ranges.clone()
  }
}

/// Opens the first range's decoder while audio starts and prebuffers, then
/// starts the audio clock once both are ready. `None` means playback never
/// began: it failed, which is reported, or was cancelled meanwhile.
fn start(
  context: &RunContext,
  video: &Arc<VideoSource>,
  ranges: &[RecordingPreviewPlaybackRange],
) -> Option<(VideoPlayback, Option<AudioPlayback>)> {
  let first_video = video.prepare(ranges[0]);
  let audio = if context.sources.audio_tracks.is_empty() {
    Ok(None)
  } else {
    audio::spawn(
      &context.sources,
      Arc::clone(&context.selected_audio),
      Arc::clone(&context.audio_volumes),
      ranges,
      context.playback_rate,
      Arc::clone(&context.cancelled),
      Arc::clone(&context.audio_child),
    )
    .map(Some)
  };
  let (video, audio) = match (first_video.ready(), audio) {
    (Ok(video), Ok(audio)) => (video, audio),
    (video, audio) => {
      let error = video.as_ref().err().or(audio.as_ref().err()).cloned();
      abandon_start(context, video.ok(), audio.ok().flatten());
      if let Some(error) = error {
        send_error(&context.event_channel, error);
      }
      return None;
    }
  };
  if context.cancelled.load(Ordering::Acquire) {
    abandon_start(context, Some(video), audio);
    return None;
  }
  if let Some(Err(error)) = audio.as_ref().map(AudioPlayback::play) {
    abandon_start(context, Some(video), audio);
    send_error(&context.event_channel, error);
    return None;
  }
  Some((video, audio))
}

fn abandon_start(context: &RunContext, video: Option<VideoPlayback>, audio: Option<AudioPlayback>) {
  if let Some(video) = video {
    let _ = video.cancel().join();
  }
  if let Some(audio) = audio {
    stop_audio(context, audio);
  }
}

fn stop_audio(context: &RunContext, audio: AudioPlayback) {
  // The audio thread waits on a full queue until playback is cancelled.
  context.cancelled.store(true, Ordering::Release);
  stop_child(&context.audio_child);
  drop(audio.stream);
  let _ = audio.thread.join();
}

pub(super) fn run(context: RunContext) {
  let ranges = ranges(&context);
  let video = VideoSource::new(&context);
  let Some((mut current_video, audio)) = start(&context, &video, &ranges) else {
    return;
  };
  let display_clock = audio.as_ref().and_then(|audio| {
    super::super::audio_visualizer_clock::install(
      &context.sources,
      &audio.clock,
      &ranges,
      context.playback_rate,
      &context.cancelled,
      &context.position_ms,
      context.start_ms,
    )
  });
  let video_clock_start = Instant::now();
  let _ = context
    .event_channel
    .send(RecordingPreviewPlayerEvent::Playing {
      position_ms: ranges[0].source_start_ms,
    });
  let elapsed_ms = || {
    audio.as_ref().map_or_else(
      || video_clock_start.elapsed().as_millis() as u64,
      |playback| (playback.clock.seconds() * 1_000.0) as u64,
    )
  };
  let mut next_video = ranges.get(1).map(|range| video.prepare(*range));
  let mut finished_videos = Vec::with_capacity(ranges.len());
  let mut output_offset_ms = 0;
  let mut failed = false;
  let mut late_frames = LateFrames::default();

  for (range_index, range) in ranges.iter().copied().enumerate() {
    let rate = effective_rate(range, context.playback_rate);
    let output_end_ms = output_offset_ms + output_duration_ms(range.duration_ms(), rate);
    while !context.cancelled.load(Ordering::Acquire) {
      let frame = match current_video.frames.recv_timeout(Duration::from_millis(16)) {
        Ok(frame) => frame,
        Err(mpsc::RecvTimeoutError::Timeout) if elapsed_ms() < output_end_ms => continue,
        Err(_) => break,
      };
      let frame_output_ms = output_offset_ms + frame.presentation_elapsed_ms;
      // Negative: early, and slept until due. Positive: drawn this late.
      let late_ms = elapsed_ms() as i64 - frame_output_ms as i64;
      if context.sources.presents_video() && late_frames.skips(late_ms) {
        continue;
      }
      while elapsed_ms() < frame_output_ms && !context.cancelled.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(2));
      }
      if context.cancelled.load(Ordering::Acquire) || elapsed_ms() >= output_end_ms {
        break;
      }
      let current = range
        .source_start_ms
        .saturating_add(source_elapsed_ms(
          elapsed_ms().saturating_sub(output_offset_ms),
          rate,
        ))
        .min(range.source_end_ms);
      if context.sources.presents_video()
        || !display_clock.as_ref().is_some_and(|clock| clock.active())
      {
        context.position_ms.store(current, Ordering::Release);
      }
      if !context.sources.presents_video() && display_clock.is_none() {
        present_audio_position(&context.sources, current);
      }
      if context.sources.presents_video() && !platform::send_frame(&context.sources, frame.payload)
      {
        failed = true;
        break;
      }
      late_frames.drew();
      let _ = context
        .event_channel
        .send(RecordingPreviewPlayerEvent::Position {
          position_ms: current,
        });
    }
    complete_range(
      &context.position_ms,
      &context.cancelled,
      failed,
      range.source_end_ms,
    );
    finished_videos.push(current_video.cancel());
    output_offset_ms = output_end_ms;
    if failed || context.cancelled.load(Ordering::Acquire) || range_index + 1 == ranges.len() {
      break;
    }
    current_video = match next_video.take().map(PreparedVideo::ready) {
      Some(Ok(playback)) => playback,
      Some(Err(error)) => {
        send_error(&context.event_channel, error);
        failed = true;
        break;
      }
      None => break,
    };
    next_video = ranges.get(range_index + 2).map(|next| video.prepare(*next));
  }

  if let Some(Ok(playback)) = next_video.map(PreparedVideo::ready) {
    finished_videos.push(playback.cancel());
  }
  for thread in finished_videos {
    let _ = thread.join();
  }
  let was_cancelled = context.cancelled.load(Ordering::Acquire);
  context.cancelled.store(true, Ordering::Release);
  stop_child(&context.video_child);
  if let Some(audio) = audio {
    stop_audio(&context, audio);
  }
  if was_cancelled || failed {
    return;
  }
  let final_end_ms = ranges
    .last()
    .map_or(context.start_ms, |range| range.source_end_ms);
  if final_end_ms < context.sources.duration_ms {
    let _ = context
      .event_channel
      .send(RecordingPreviewPlayerEvent::RangeEnded {
        position_ms: final_end_ms,
      });
  } else {
    context
      .position_ms
      .store(context.sources.duration_ms, Ordering::Release);
    let _ = context
      .event_channel
      .send(RecordingPreviewPlayerEvent::Ended);
  }
}

#[cfg(test)]
mod tests {
  use super::{complete_range, output_duration_ms};
  use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

  #[test]
  fn playback_duration_scales_by_rate() {
    assert_eq!(output_duration_ms(1_000, 0.5), 2_000);
    assert_eq!(output_duration_ms(1_000, 2.0), 500);
  }

  #[test]
  fn cancelling_playback_keeps_the_last_presented_position() {
    let position = AtomicU64::new(2_400);
    complete_range(&position, &AtomicBool::new(true), false, 8_000);
    assert_eq!(position.load(Ordering::Acquire), 2_400);
  }
}
