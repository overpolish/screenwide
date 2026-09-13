// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows recording audio.
//!
//! Media Foundation's MP4 sink accepts only one audio stream. Microphone and
//! system audio are therefore captured as timestamped float PCM sidecars and
//! muxed into independent AAC streams after the video sink is finalized. The
//! H.264 video is stream-copied, so this adds no video decode or CPU render.

#[path = "audio/mux.rs"]
mod mux;
#[path = "audio/raw.rs"]
mod raw;
#[path = "audio/system.rs"]
mod system;
use mux::mux_file;
use system::start_system;

use std::{
  fs::{self, File},
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  process::Command,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, OnceLock,
  },
  thread::{self, JoinHandle},
  time::{Duration, Instant},
};

use cpal::Stream;
use wasapi::{
  initialize_mta, AudioCaptureClient, AudioClient, DeviceEnumerator, Direction, Handle, SampleType,
  StreamMode, WaveFormat,
};

use crate::recording::{
  encoding::FailureReport, microphone, monitor::RecordingMonitor, SystemAudioSelection,
};

const SYSTEM_SAMPLE_RATE: u32 = 48_000;
const SYSTEM_CHANNELS: u16 = 2;

struct Packet {
  captured_at: Instant,
  samples: Vec<f32>,
}

#[derive(Clone)]
struct RawSource {
  channels: u16,
  first_offset_ms: u64,
  path: PathBuf,
  sample_rate: u32,
}

struct RawSink {
  cleanup_on_drop: bool,
  path: PathBuf,
  sender: Option<mpsc::SyncSender<Packet>>,
  worker: Option<JoinHandle<Result<RawSource, String>>>,
}

struct MicrophoneCapture {
  sink: RawSink,
  stream: Stream,
}

struct SystemCapture {
  sink: RawSink,
  stop: Arc<AtomicBool>,
  thread: Option<JoinHandle<()>>,
}

pub(super) struct AudioCaptures {
  microphone: Option<MicrophoneCapture>,
  paused: Arc<AtomicBool>,
  system: Vec<SystemCapture>,
}

pub(super) struct AudioFiles {
  pub(super) has_microphone: bool,
  pub(super) has_system_audio: bool,
  microphone: Option<RawSource>,
  system: Vec<RawSource>,
}

impl AudioCaptures {
  pub(super) fn start(
    microphone_id: Option<&str>,
    system: &SystemAudioSelection,
    origin: Arc<OnceLock<Instant>>,
    monitor: Arc<RecordingMonitor>,
    on_failure: FailureReport,
    video_path: &Path,
  ) -> Result<Self, String> {
    let paused = Arc::new(AtomicBool::new(false));
    let microphone = microphone_id
      .map(|device_id| {
        start_microphone(
          device_id,
          sidecar_path(video_path, "microphone", 0),
          Arc::clone(&origin),
          Arc::clone(&paused),
          Arc::clone(&monitor),
          Arc::clone(&on_failure),
        )
      })
      .transpose()?;

    let mut captures = Vec::new();
    if system.enabled {
      let process_ids = if system.process_ids.is_empty() {
        vec![None]
      } else {
        let mut ids = system.process_ids.clone();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter().map(Some).collect()
      };
      for (index, process_id) in process_ids.into_iter().enumerate() {
        captures.push(start_system(
          process_id,
          sidecar_path(video_path, "system", index),
          Arc::clone(&origin),
          Arc::clone(&paused),
          Arc::clone(&monitor),
          Arc::clone(&on_failure),
        )?);
      }
    }
    Ok(Self {
      microphone,
      paused,
      system: captures,
    })
  }

  pub(super) fn pause(&self) {
    self.paused.store(true, Ordering::Release);
  }

  pub(super) fn resume(&self) {
    self.paused.store(false, Ordering::Release);
  }

  pub(super) fn finish(mut self) -> Result<AudioFiles, String> {
    let microphone = if let Some(capture) = self.microphone.take() {
      drop(capture.stream);
      Some(capture.sink.finish()?)
    } else {
      None
    };
    let mut system = Vec::with_capacity(self.system.len());
    for mut capture in self.system.drain(..) {
      capture.stop.store(true, Ordering::Release);
      if let Some(thread) = capture.thread.take() {
        let _ = thread.join();
      }
      system.push(capture.sink.finish()?);
    }
    Ok(AudioFiles {
      has_microphone: microphone.is_some(),
      has_system_audio: !system.is_empty(),
      microphone,
      system,
    })
  }
}

fn sidecar_path(video: &Path, kind: &str, index: usize) -> PathBuf {
  let stem = video
    .file_stem()
    .and_then(|value| value.to_str())
    .unwrap_or("recording");
  video.with_file_name(format!("{stem}.{kind}-{index}.f32"))
}

fn start_microphone(
  device_id: &str,
  path: PathBuf,
  origin: Arc<OnceLock<Instant>>,
  paused: Arc<AtomicBool>,
  monitor: Arc<RecordingMonitor>,
  on_failure: FailureReport,
) -> Result<MicrophoneCapture, String> {
  let source = microphone::Source::resolve(device_id)?;
  let format = source.format();
  let sink = RawSink::start(path, format.sample_rate, format.channels, origin)?;
  let sender = sink.sender()?;
  let callback_monitor = Arc::clone(&monitor);
  let stream = source.start(
    Arc::new(move |buffer| {
      callback_monitor.send_microphone(&buffer.samples);
      if !paused.load(Ordering::Acquire) {
        let _ = sender.try_send(Packet {
          captured_at: buffer.captured_at,
          samples: buffer.samples,
        });
      }
    }),
    on_failure,
  )?;
  Ok(MicrophoneCapture { sink, stream })
}

pub(super) fn mux(video: &Path, duration_ms: u64, files: AudioFiles) -> Result<(), String> {
  mux_file(video, duration_ms, files, true)
}

pub(super) fn mux_audio_only(
  output_path: &Path,
  duration_ms: u64,
  files: AudioFiles,
) -> Result<(), String> {
  mux_file(output_path, duration_ms, files, false)
}
