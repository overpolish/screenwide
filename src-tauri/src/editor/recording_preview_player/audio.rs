// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) mod clock;
mod filter;

#[path = "audio/output.rs"]
mod output;
use output::output_stream;

use std::{
  collections::VecDeque,
  io::Read,
  process::{Child, Command, Stdio},
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
  let stream_indices = sources
    .audio_tracks
    .iter()
    .map(|track| track.stream_index)
    .collect::<Vec<_>>();
  let track_count = stream_indices.len();
  let (stream, clock, config) = output_stream(
    Arc::clone(&queue),
    Arc::clone(&selected_audio),
    Arc::clone(&audio_volumes),
    stream_indices,
  )?;
  let mut process = Command::new(media_preview::ffmpeg_path());
  process
    .args(args(sources, ranges, &config, playback_rate))
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
        for chunk in bytes[..count].chunks_exact(4) {
          queue.push_back(f32::from_le_bytes(chunk.try_into().unwrap_or([0; 4])));
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
#[path = "audio_tests.rs"]
mod tests;
