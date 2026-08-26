// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Receiver},
    Arc, Mutex, RwLock,
  },
  time::{Duration, Instant},
};

use tauri::ipc::Channel;

use super::{audio, platform, send_error, stop_child};
use crate::exports::recording_preview_player::video::VideoFrame;
use crate::exports::recording_preview_player::{
  AudioTrackVolume, PlayerSources, RecordingPreviewPlaybackRange, RecordingPreviewPlayerEvent,
};

pub(super) struct RunContext {
  pub audio_child: Arc<Mutex<Option<Child>>>,
  pub audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  pub cancelled: Arc<AtomicBool>,
  pub event_channel: Channel<RecordingPreviewPlayerEvent>,
  pub playback_end_ms: Option<u64>,
  pub playback_factors: Vec<f64>,
  pub playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  pub position_ms: Arc<AtomicU64>,
  pub selected_audio: Arc<RwLock<Vec<usize>>>,
  pub sources: PlayerSources,
  pub start_ms: u64,
  pub video_child: Arc<Mutex<Option<Child>>>,
}

struct VideoPlayback {
  cancelled: Arc<AtomicBool>,
  frames: Receiver<VideoFrame>,
  thread: std::thread::JoinHandle<()>,
}

impl VideoPlayback {
  fn spawn(context: &RunContext, start_ms: u64) -> Result<Self, String> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let (sender, frames) = mpsc::sync_channel(3);
    let thread = platform::spawn_video(
      &context.sources,
      &context.playback_factors,
      start_ms,
      false,
      Arc::clone(&cancelled),
      Arc::clone(&context.video_child),
      sender,
    )?;
    Ok(Self {
      cancelled,
      frames,
      thread,
    })
  }

  fn stop(self) {
    self.cancelled.store(true, Ordering::Release);
    let _ = self.thread.join();
  }
}

const fn presentation_elapsed_ms(frame_timestamp_ms: u64, start_ms: u64) -> u64 {
  frame_timestamp_ms.saturating_sub(start_ms)
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
    }]
  } else {
    context.playback_ranges.clone()
  }
}

pub(super) fn run(context: RunContext) {
  let ranges = ranges(&context);
  let mut current_video = match VideoPlayback::spawn(&context, ranges[0].source_start_ms) {
    Ok(playback) => playback,
    Err(error) => return send_error(&context.event_channel, error),
  };
  let audio = if context.sources.audio_tracks.is_empty() {
    None
  } else {
    match audio::spawn(
      &context.sources,
      Arc::clone(&context.selected_audio),
      Arc::clone(&context.audio_volumes),
      &ranges,
      Arc::clone(&context.cancelled),
      Arc::clone(&context.audio_child),
    ) {
      Ok(playback) => Some(playback),
      Err(error) => {
        current_video.stop();
        return send_error(&context.event_channel, error);
      }
    }
  };
  let video_clock_start = Instant::now();
  let _ = context
    .event_channel
    .send(RecordingPreviewPlayerEvent::Playing {
      position_ms: ranges[0].source_start_ms,
    });
  let elapsed_ms = || {
    audio.as_ref().map_or_else(
      || video_clock_start.elapsed().as_millis() as u64,
      |playback| {
        playback.played_frames.load(Ordering::Acquire) * 1_000 / u64::from(playback.sample_rate)
      },
    )
  };
  let mut next_video = ranges
    .get(1)
    .map(|range| VideoPlayback::spawn(&context, range.source_start_ms));
  let mut output_offset_ms = 0;
  let mut failed = false;

  for (range_index, range) in ranges.iter().copied().enumerate() {
    let output_end_ms = output_offset_ms + range.duration_ms();
    while !context.cancelled.load(Ordering::Acquire) {
      let frame = match current_video.frames.recv_timeout(Duration::from_millis(16)) {
        Ok(frame) => frame,
        Err(mpsc::RecvTimeoutError::Timeout) if elapsed_ms() < output_end_ms => continue,
        Err(_) => break,
      };
      let frame_output_ms =
        output_offset_ms + presentation_elapsed_ms(frame.timestamp_ms, range.source_start_ms);
      while elapsed_ms() < frame_output_ms && !context.cancelled.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(2));
      }
      if context.cancelled.load(Ordering::Acquire) || elapsed_ms() >= output_end_ms {
        break;
      }
      let current = range
        .source_start_ms
        .saturating_add(elapsed_ms().saturating_sub(output_offset_ms))
        .min(range.source_end_ms);
      context.position_ms.store(current, Ordering::Release);
      if !platform::send_frame(&context.sources, frame.payload) {
        failed = true;
        break;
      }
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
    current_video.stop();
    output_offset_ms = output_end_ms;
    if failed || context.cancelled.load(Ordering::Acquire) || range_index + 1 == ranges.len() {
      break;
    }
    current_video = match next_video.take() {
      Some(Ok(playback)) => playback,
      Some(Err(error)) => {
        send_error(&context.event_channel, error);
        failed = true;
        break;
      }
      None => break,
    };
    next_video = ranges
      .get(range_index + 2)
      .map(|next| VideoPlayback::spawn(&context, next.source_start_ms));
  }

  if let Some(Ok(playback)) = next_video {
    playback.stop();
  }
  let was_cancelled = context.cancelled.load(Ordering::Acquire);
  context.cancelled.store(true, Ordering::Release);
  stop_child(&context.video_child);
  stop_child(&context.audio_child);
  if let Some(audio) = audio {
    drop(audio.stream);
    let _ = audio.thread.join();
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
  use super::{complete_range, presentation_elapsed_ms};
  use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

  #[test]
  fn playback_follows_media_timestamps_after_a_seek() {
    assert_eq!(presentation_elapsed_ms(7_126, 5_000), 2_126);
    assert_eq!(presentation_elapsed_ms(4_999, 5_000), 0);
  }

  #[test]
  fn cancelling_playback_keeps_the_last_presented_position() {
    let position = AtomicU64::new(2_400);
    complete_range(&position, &AtomicBool::new(true), false, 8_000);
    assert_eq!(position.load(Ordering::Acquire), 2_400);
  }
}
