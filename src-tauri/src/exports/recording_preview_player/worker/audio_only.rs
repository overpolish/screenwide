// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, RwLock,
  },
  time::Duration,
};

use tauri::ipc::Channel;

use super::{audio, send_error, stop_child, PlaybackMode};
use crate::exports::recording_preview_player::{
  PlayerSources, RecordingPreviewPlaybackRange, RecordingPreviewPlayerEvent,
};
use crate::exports::AudioTrackVolume;

pub(super) struct RunContext {
  pub(super) audio_child: Arc<Mutex<Option<Child>>>,
  pub(super) cancelled: Arc<AtomicBool>,
  pub(super) event_channel: Channel<RecordingPreviewPlayerEvent>,
  pub(super) mode: PlaybackMode,
  pub(super) playback_end_ms: Option<u64>,
  pub(super) playback_rate: f64,
  pub(super) playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  pub(super) position_ms: Arc<AtomicU64>,
  pub(super) request_id: u64,
  pub(super) selected_audio: Arc<RwLock<Vec<usize>>>,
  pub(super) audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  pub(super) sources: PlayerSources,
  pub(super) start_ms: u64,
}

const fn plays_audio(mode: PlaybackMode) -> bool {
  matches!(mode, PlaybackMode::Playing)
}

fn output_duration_ms(source_duration_ms: u64, playback_rate: f64) -> u64 {
  ((source_duration_ms as f64) / playback_rate).ceil() as u64
}

fn effective_rate(range: RecordingPreviewPlaybackRange, global_rate: f64) -> f64 {
  range.playback_rate * global_rate
}

fn position_at_elapsed(
  ranges: &[RecordingPreviewPlaybackRange],
  elapsed_ms: u64,
  playback_rate: f64,
  fallback_ms: u64,
) -> u64 {
  let mut remaining = elapsed_ms;
  for range in ranges {
    let rate = effective_rate(*range, playback_rate);
    let output_duration = output_duration_ms(range.duration_ms(), rate);
    if remaining < output_duration {
      return range
        .source_start_ms
        .saturating_add(((remaining as f64) * rate).round() as u64)
        .min(range.source_end_ms);
    }
    remaining = remaining.saturating_sub(output_duration);
  }
  ranges
    .last()
    .map_or(fallback_ms, |range| range.source_end_ms)
}

pub(super) fn run(context: RunContext) {
  let RunContext {
    audio_child,
    cancelled,
    event_channel,
    mode,
    playback_end_ms,
    playback_rate,
    playback_ranges,
    position_ms,
    request_id,
    selected_audio,
    audio_volumes,
    sources,
    start_ms,
  } = context;
  if !plays_audio(mode) {
    position_ms.store(start_ms, Ordering::Release);
    let _ = event_channel.send(RecordingPreviewPlayerEvent::Ready {
      position_ms: start_ms,
      request_id,
    });
    return;
  }
  let ranges = if playback_ranges.is_empty() {
    vec![RecordingPreviewPlaybackRange {
      source_start_ms: start_ms,
      source_end_ms: playback_end_ms
        .unwrap_or(sources.duration_ms)
        .min(sources.duration_ms)
        .max(start_ms),
      playback_rate: 1.0,
    }]
  } else {
    playback_ranges
  };
  let audio = match audio::spawn(
    &sources,
    selected_audio,
    audio_volumes,
    &ranges,
    playback_rate,
    Arc::clone(&cancelled),
    Arc::clone(&audio_child),
  ) {
    Ok(audio) => audio,
    Err(error) => return send_error(&event_channel, error),
  };
  if cancelled.load(Ordering::Acquire) {
    stop_child(&audio_child);
    drop(audio.stream);
    let _ = audio.thread.join();
    return;
  }
  let _ = event_channel.send(RecordingPreviewPlayerEvent::Playing {
    position_ms: start_ms,
  });
  let output_duration_ms = ranges
    .iter()
    .map(|range| output_duration_ms(range.duration_ms(), effective_rate(*range, playback_rate)))
    .sum::<u64>();
  while !cancelled.load(Ordering::Acquire) {
    let elapsed =
      audio.played_frames.load(Ordering::Acquire) * 1_000 / u64::from(audio.sample_rate);
    if elapsed >= output_duration_ms {
      position_ms.store(
        ranges.last().map_or(start_ms, |range| range.source_end_ms),
        Ordering::Release,
      );
      break;
    }
    let current = position_at_elapsed(&ranges, elapsed, playback_rate, start_ms);
    position_ms.store(current, Ordering::Release);
    let _ = event_channel.send(RecordingPreviewPlayerEvent::Position {
      position_ms: current,
    });
    std::thread::sleep(Duration::from_millis(
      output_duration_ms.saturating_sub(elapsed).clamp(1, 16),
    ));
  }
  cancelled.store(true, Ordering::Release);
  stop_child(&audio_child);
  drop(audio.stream);
  let _ = audio.thread.join();
  let final_end_ms = ranges.last().map_or(start_ms, |range| range.source_end_ms);
  if final_end_ms < sources.duration_ms && position_ms.load(Ordering::Acquire) >= final_end_ms {
    let _ = event_channel.send(RecordingPreviewPlayerEvent::RangeEnded {
      position_ms: final_end_ms,
    });
  } else if position_ms.load(Ordering::Acquire) >= sources.duration_ms.saturating_sub(50) {
    position_ms.store(sources.duration_ms, Ordering::Release);
    let _ = event_channel.send(RecordingPreviewPlayerEvent::Ended);
  }
}

#[cfg(test)]
mod tests {
  use super::{output_duration_ms, plays_audio, position_at_elapsed, PlaybackMode};
  use crate::exports::recording_preview_player::RecordingPreviewPlaybackRange;

  #[test]
  fn maps_scaled_elapsed_time_across_deleted_gaps() {
    let ranges = [
      RecordingPreviewPlaybackRange {
        source_end_ms: 1_000,
        source_start_ms: 0,
        playback_rate: 1.0,
      },
      RecordingPreviewPlaybackRange {
        source_end_ms: 3_000,
        source_start_ms: 2_000,
        playback_rate: 1.0,
      },
    ];
    assert_eq!(output_duration_ms(1_000, 2.0), 500);
    assert_eq!(position_at_elapsed(&ranges, 250, 2.0, 0), 500);
    assert_eq!(position_at_elapsed(&ranges, 500, 2.0, 0), 2_000);
    assert_eq!(position_at_elapsed(&ranges, 750, 2.0, 0), 2_500);
  }

  #[test]
  fn mixed_range_rates_map_elapsed_time_independently() {
    let ranges = [
      RecordingPreviewPlaybackRange {
        source_end_ms: 1_000,
        source_start_ms: 0,
        playback_rate: 2.0,
      },
      RecordingPreviewPlaybackRange {
        source_end_ms: 3_000,
        source_start_ms: 2_000,
        playback_rate: 0.5,
      },
    ];
    assert_eq!(position_at_elapsed(&ranges, 500, 1.0, 0), 2_000);
    assert_eq!(position_at_elapsed(&ranges, 1_500, 1.0, 0), 2_500);
  }

  #[test]
  fn paused_and_scrubbed_audio_only_previews_do_not_play() {
    assert!(!plays_audio(PlaybackMode::Still));
    assert!(!plays_audio(PlaybackMode::InteractiveStill));
    assert!(plays_audio(PlaybackMode::Playing));
  }
}
