// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Which writer a capture's frames go to.
pub(super) enum WriterKind {
  Recording,
  Replay {
    keyframes: replay::KeyframeLink,
    leads: bool,
    length: Duration,
  },
}

pub(super) fn spawn_writer(
  name: &str,
  config: WriterConfig,
  kind: WriterKind,
) -> Result<WriterSpawn, String> {
  let (commands, command_rx) = mpsc::sync_channel(8);
  let (initialized_tx, initialized) = mpsc::channel();
  let (first_frame_tx, first_frame) = mpsc::channel();
  let worker = std::thread::Builder::new()
    .name(name.to_owned())
    .spawn(move || match kind {
      WriterKind::Recording => writer::run(config, command_rx, initialized_tx, first_frame_tx),
      WriterKind::Replay {
        keyframes,
        leads,
        length,
      } => replay::run(
        replay::ReplayWriterConfig {
          keyframes,
          leads,
          length,
          writer: config,
        },
        command_rx,
        initialized_tx,
        first_frame_tx,
      ),
    })
    .map_err(|error| error.to_string())?;
  initialized
    .recv()
    .map_err(|_| "The recording writer stopped during startup".to_owned())??;
  Ok((commands, first_frame, worker))
}

pub(super) fn both_first_frames(
  primary: mpsc::Receiver<Result<(), String>>,
  camera: mpsc::Receiver<Result<(), String>>,
) -> mpsc::Receiver<Result<(), String>> {
  let (ready, combined) = mpsc::channel();
  std::thread::spawn(move || {
    let result = primary
      .recv()
      .map_err(|_| "The primary recording stopped before its first frame".to_owned())
      .and_then(|result| result)
      .and_then(|()| {
        camera
          .recv()
          .map_err(|_| "The camera stopped before its first frame".to_owned())?
      });
    let _ = ready.send(result);
  });
  combined
}

pub(super) fn begin_audio_only(
  microphone_id: Option<&str>,
  system_audio: &crate::recording::SystemAudioSelection,
  monitor: Arc<crate::recording::monitor::RecordingMonitor>,
  on_failure: crate::recording::encoding::FailureReport,
  path: std::path::PathBuf,
  replay_horizon_ns: Option<i64>,
) -> Result<CaptureStart, String> {
  if microphone_id.is_none() && !system_audio.enabled {
    return Err("Select a microphone or system audio source".to_owned());
  }
  let started = Instant::now();
  let timeline_origin = Arc::new(OnceLock::new());
  let _ = timeline_origin.set(started);
  let audio = audio::AudioCaptures::start(
    microphone_id,
    system_audio,
    Arc::clone(&timeline_origin),
    monitor,
    on_failure,
    audio_destination(&path, replay_horizon_ns),
  )?;
  let (ready, first_frame) = mpsc::channel();
  let _ = ready.send(Ok(()));
  Ok(CaptureStart {
    cursor_source: None,
    first_frame,
    session: CaptureSession {
      audio: Some(audio),
      audio_only_clock: Some(AudioOnlyClock::new(started)),
      audio_only_path: Some(path),
      camera: None,
      captures: Vec::new(),
      commands: None,
      primary_camera: None,
      stopped_at: Arc::new(OnceLock::new()),
      worker: None,
    },
    source_scale_factor: 1.0,
    timeline_origin,
  })
}

/// Where a capture's audio goes: a recording's sidecars beside `path`, or
/// a replay buffer's memory reaching `replay_horizon_ns` back.
pub(super) fn audio_destination(
  path: &std::path::Path,
  replay_horizon_ns: Option<i64>,
) -> audio::AudioDestination<'_> {
  match replay_horizon_ns {
    Some(horizon_ns) => audio::AudioDestination::Rings { horizon_ns },
    None => audio::AudioDestination::Sidecars(path),
  }
}
