// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod audio_only;
mod ranged;

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc, Mutex, RwLock,
  },
};

use tauri::ipc::Channel;

use super::{
  audio, platform, AudioTrackVolume, PlayerSources, PreviewAudioSettings,
  RecordingPreviewPlaybackRange, RecordingPreviewPlayerEvent,
};

#[derive(Clone, Copy)]
pub(super) enum PlaybackMode {
  InteractiveStill,
  Playing,
  Still,
}

pub(super) struct WorkerLaunch {
  pub audio: PreviewAudioSettings,
  pub mode: PlaybackMode,
  pub playback_factors: Vec<f64>,
  pub playback_end_ms: Option<u64>,
  pub playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  pub request_id: u64,
  pub start_ms: u64,
}

pub(super) struct PreviewPlayerWorker {
  audio_child: Arc<Mutex<Option<Child>>>,
  cancelled: Arc<AtomicBool>,
  position_ms: Arc<AtomicU64>,
  selected_audio: Arc<RwLock<Vec<usize>>>,
  audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  thread: Option<std::thread::JoinHandle<()>>,
  video_child: Arc<Mutex<Option<Child>>>,
}

fn stop_child(child: &Arc<Mutex<Option<Child>>>) {
  if let Ok(mut child) = child.lock() {
    if let Some(mut process) = child.take() {
      let _ = process.kill();
      let _ = process.wait();
    }
  }
}

fn send_error(channel: &Channel<RecordingPreviewPlayerEvent>, message: String) {
  let _ = channel.send(RecordingPreviewPlayerEvent::Error { message });
}

struct RunContext {
  audio_child: Arc<Mutex<Option<Child>>>,
  audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  cancelled: Arc<AtomicBool>,
  event_channel: Channel<RecordingPreviewPlayerEvent>,
  mode: PlaybackMode,
  playback_factors: Vec<f64>,
  playback_end_ms: Option<u64>,
  playback_ranges: Vec<RecordingPreviewPlaybackRange>,
  position_ms: Arc<AtomicU64>,
  request_id: u64,
  selected_audio: Arc<RwLock<Vec<usize>>>,
  sources: PlayerSources,
  start_ms: u64,
  video_child: Arc<Mutex<Option<Child>>>,
}

fn run(context: RunContext) {
  let RunContext {
    sources,
    selected_audio,
    audio_volumes,
    start_ms,
    mode,
    event_channel,
    cancelled,
    playback_factors,
    playback_end_ms,
    playback_ranges,
    position_ms,
    request_id,
    video_child,
    audio_child,
  } = context;
  if sources.layout.panes.is_empty() {
    return audio_only::run(audio_only::RunContext {
      audio_child,
      cancelled,
      event_channel,
      mode,
      playback_end_ms,
      playback_ranges,
      position_ms,
      request_id,
      selected_audio,
      audio_volumes,
      sources,
      start_ms,
    });
  }
  if matches!(mode, PlaybackMode::Still | PlaybackMode::InteractiveStill) {
    let (frame_tx, frame_rx) = mpsc::sync_channel(3);
    let video_thread = match platform::spawn_video(
      &sources,
      &playback_factors,
      start_ms,
      true,
      Arc::clone(&cancelled),
      Arc::clone(&video_child),
      frame_tx,
    ) {
      Ok(thread) => thread,
      Err(error) => {
        send_error(&event_channel, error);
        return;
      }
    };
    if let Ok(frame) = frame_rx.recv() {
      if !cancelled.load(Ordering::Acquire) && platform::send_frame(&sources, frame.payload) {
        position_ms.store(start_ms, Ordering::Release);
        let _ = event_channel.send(RecordingPreviewPlayerEvent::Ready {
          position_ms: start_ms,
          request_id,
        });
      }
    }
    stop_child(&video_child);
    let _ = video_thread.join();
    return;
  }
  ranged::run(ranged::RunContext {
    audio_child,
    audio_volumes,
    cancelled,
    event_channel,
    playback_end_ms,
    playback_factors,
    playback_ranges,
    position_ms,
    selected_audio,
    sources,
    start_ms,
    video_child,
  });
}

impl PreviewPlayerWorker {
  pub(super) fn spawn(
    sources: PlayerSources,
    launch: WorkerLaunch,
    event_channel: Channel<RecordingPreviewPlayerEvent>,
  ) -> Result<Self, String> {
    let WorkerLaunch {
      audio,
      mode,
      playback_factors,
      playback_end_ms,
      playback_ranges,
      request_id,
      start_ms,
    } = launch;
    let cancelled = Arc::new(AtomicBool::new(false));
    let position_ms = Arc::new(AtomicU64::new(start_ms));
    let video_child = Arc::new(Mutex::new(None));
    let audio_child = Arc::new(Mutex::new(None));
    let selected_audio = Arc::new(RwLock::new(audio.enabled_stream_indices));
    let audio_volumes = Arc::new(RwLock::new(audio.audio_track_volumes));
    let thread = std::thread::Builder::new()
      .name("recording-preview-player".to_owned())
      .spawn({
        let cancelled = Arc::clone(&cancelled);
        let position_ms = Arc::clone(&position_ms);
        let video_child = Arc::clone(&video_child);
        let audio_child = Arc::clone(&audio_child);
        let selected_audio = Arc::clone(&selected_audio);
        let audio_volumes = Arc::clone(&audio_volumes);
        move || {
          run(RunContext {
            audio_child,
            cancelled,
            event_channel,
            mode,
            playback_factors,
            playback_end_ms,
            playback_ranges,
            position_ms,
            request_id,
            selected_audio,
            audio_volumes,
            sources,
            start_ms,
            video_child,
          });
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      audio_child,
      cancelled,
      position_ms,
      selected_audio,
      audio_volumes,
      thread: Some(thread),
      video_child,
    })
  }

  pub(super) fn select_audio(&self, enabled_stream_indices: Vec<usize>) -> Result<(), String> {
    *self
      .selected_audio
      .write()
      .map_err(|_| "The preview audio selection is unavailable".to_owned())? =
      enabled_stream_indices;
    Ok(())
  }

  pub(super) fn set_audio_volumes(
    &self,
    audio_track_volumes: Vec<AudioTrackVolume>,
  ) -> Result<(), String> {
    *self
      .audio_volumes
      .write()
      .map_err(|_| "The preview audio volumes are unavailable".to_owned())? = audio_track_volumes;
    Ok(())
  }

  /// Stops the worker's decode processes and gates its frame sends without
  /// joining anything: the kills are milliseconds, while the joins - the
  /// ffmpeg audio child, the CoreAudio output stream, the decode threads - are
  /// slow enough to freeze whichever thread waits on them. Callers signal here
  /// and join through `cancel` off the main thread.
  pub(super) fn signal_cancel(&self) {
    self.cancelled.store(true, Ordering::Release);
    stop_child(&self.video_child);
    stop_child(&self.audio_child);
  }

  /// The last position the worker presented, readable without joining it.
  pub(super) fn position(&self) -> u64 {
    self.position_ms.load(Ordering::Acquire)
  }

  pub(super) fn cancel(mut self) -> u64 {
    self.cancelled.store(true, Ordering::Release);
    stop_child(&self.video_child);
    stop_child(&self.audio_child);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
    self.position_ms.load(Ordering::Acquire)
  }
}
