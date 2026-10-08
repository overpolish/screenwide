// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) mod clock;
mod filter;

mod output;
use output::{output_stream, Mix};

use std::{
  collections::VecDeque,
  io::Read,
  process::{Child, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
  },
  time::{Duration, Instant},
};

use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, SampleFormat, SizedSample, Stream, StreamConfig,
};

use self::{clock::AudioClock, filter::args};
use super::{PlayerSources, RecordingPreviewPlaybackRange};
use crate::editor::{media_preview, AudioTrackVolume};

/// One channel of what FFmpeg decodes for the preview: a recorded track, or
/// that track's cleaned file, decoded beside it so the noise switch can
/// change which is heard while it plays.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Channel {
  pub stream_index: usize,
  pub cleaned: Option<std::path::PathBuf>,
}

fn channels(sources: &PlayerSources) -> Vec<Channel> {
  let noise = sources
    .noise
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let tracks = sources.audio_tracks.iter().map(|track| Channel {
    cleaned: None,
    stream_index: track.stream_index,
  });
  let cleaned = noise
    .cleaned
    .iter()
    .filter(|(stream, _)| {
      sources
        .audio_tracks
        .iter()
        .any(|track| track.stream_index == *stream)
    })
    .map(|(stream, path)| Channel {
      cleaned: Some(path.clone()),
      stream_index: *stream,
    });
  tracks.chain(cleaned).collect()
}

const MAX_QUEUED_SECONDS: usize = 2;
const PREBUFFER_MILLISECONDS: usize = 120;

pub(super) struct AudioPlayback {
  pub clock: Arc<AudioClock>,
  pub stream: Stream,
  pub thread: std::thread::JoinHandle<()>,
}

pub(super) fn spawn(
  sources: &PlayerSources,
  selected_audio: Arc<RwLock<Vec<usize>>>,
  audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  ranges: &[RecordingPreviewPlaybackRange],
  playback_rate: f64,
  cancelled: Arc<AtomicBool>,
  child: Arc<Mutex<Option<Child>>>,
) -> Result<AudioPlayback, String> {
  let queue = Arc::new(Mutex::new(VecDeque::new()));
  let channels = channels(sources);
  let track_count = channels.len();
  let (stream, clock, config) = output_stream(
    Arc::clone(&queue),
    Mix {
      audio_volumes: Arc::clone(&audio_volumes),
      channels: channels.clone(),
      noise: Arc::clone(&sources.noise),
      selected_audio: Arc::clone(&selected_audio),
    },
  )?;
  let mut process = media_preview::ffmpeg_command();
  process
    .args(args(sources, &channels, ranges, &config, playback_rate))
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
  let mut process = process
    .spawn()
    .map_err(|error| format!("FFmpeg could not start preview audio: {error}"))?;
  let mut stdout = process
    .stdout
    .take()
    .ok_or_else(|| "FFmpeg did not expose preview audio".to_owned())?;
  *child
    .lock()
    .map_err(|_| "The preview audio process is unavailable".to_owned())? = Some(process);
  let maximum = config.sample_rate as usize * track_count * MAX_QUEUED_SECONDS;
  let thread_queue = Arc::clone(&queue);
  let thread_cancelled = Arc::clone(&cancelled);
  let thread = std::thread::Builder::new()
    .name("recording-preview-audio".to_owned())
    .spawn(move || {
      let mut bytes = vec![0_u8; 16 * 1_024];
      while !thread_cancelled.load(Ordering::Acquire) {
        let count = match stdout.read(&mut bytes) {
          Ok(0) | Err(_) => break,
          Ok(count) => count - count % 4,
        };
        let mut queue = thread_queue
          .lock()
          .unwrap_or_else(|value| value.into_inner());
        for chunk in bytes[..count].as_chunks::<4>().0 {
          queue.push_back(f32::from_le_bytes(*chunk));
        }
        drop(queue);
        while thread_queue
          .lock()
          .unwrap_or_else(|value| value.into_inner())
          .len()
          > maximum
          && !thread_cancelled.load(Ordering::Acquire)
        {
          std::thread::sleep(Duration::from_millis(5));
        }
      }
    })
    .map_err(|error| error.to_string())?;

  let minimum = config.sample_rate as usize * track_count * PREBUFFER_MILLISECONDS / 1_000;
  while queue
    .lock()
    .unwrap_or_else(|value| value.into_inner())
    .len()
    < minimum
    && !cancelled.load(Ordering::Acquire)
    && !thread.is_finished()
  {
    std::thread::sleep(Duration::from_millis(5));
  }
  stream.play().map_err(|error| error.to_string())?;
  Ok(AudioPlayback {
    clock,
    stream,
    thread,
  })
}

#[cfg(test)]
mod tests;
