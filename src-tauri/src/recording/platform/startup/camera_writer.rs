// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::replay::{KeyframeLink, ReplayWriterConfig};
use super::super::*;
use super::writer_thread::{spawn_replay_writer, spawn_writer, WriterThread};
use super::Sink;
use super::{camera::CameraSpec, session::CameraObjects, writer::VideoSource};
use crate::recording::encoding::FailureReport;
use crate::recording::monitor::RecordingMonitor;

pub(super) struct CameraWriterSetup {
  pub first_frame: Option<Receiver<Result<(), String>>>,
  pub primary_spec: Option<CameraSpec>,
  pub secondary: Option<CameraObjects>,
}

pub(super) struct CameraWriterRequest<'a> {
  pub camera_flipped: bool,
  pub camera_path: Option<PathBuf>,
  pub camera_primary: bool,
  /// The primary video's keyframes, which a replayed camera follows.
  pub keyframes: KeyframeLink,
  pub monitor: &'a Arc<RecordingMonitor>,
  pub on_failure: &'a FailureReport,
  pub sink: Sink,
  pub spec: Option<CameraSpec>,
  pub timeline_origin: &'a Arc<OnceLock<Instant>>,
}

pub(super) fn prepare(request: CameraWriterRequest<'_>) -> Result<CameraWriterSetup, String> {
  let CameraWriterRequest {
    camera_flipped,
    camera_path,
    camera_primary,
    keyframes,
    monitor,
    on_failure,
    sink,
    spec,
    timeline_origin,
  } = request;
  let Some(spec) = spec else {
    return Ok(CameraWriterSetup {
      first_frame: None,
      primary_spec: None,
      secondary: None,
    });
  };
  if camera_primary {
    return Ok(CameraWriterSetup {
      first_frame: None,
      primary_spec: Some(spec),
      secondary: None,
    });
  }

  let stats = Arc::new(CaptureStats::default());
  // Both concurrent video writers use HEVC so VideoToolbox can keep
  // independent hardware-backed sessions for multi-video capture on macOS.
  let (path, thread) = match sink {
    Sink::Movie => {
      let path = camera_path.ok_or_else(|| "The camera has nowhere to record".to_owned())?;
      let thread = spawn_writer(
        WriterConfig {
          path: path.clone(),
          width: spec.width,
          height: spec.height,
          fps: spec.fps,
          encoder: VideoEncoder::Hevc,
          system_audio: false,
          microphone_format: None,
          stats: Arc::clone(&stats),
          on_failure: Arc::clone(on_failure),
          container: Container::quicktime_fragmented(),
          primary_video: false,
          source: VideoSource::Camera,
          timeline_origin: Arc::clone(timeline_origin),
        },
        "screenwide-camera-writer",
      )?;
      (Some(path), thread)
    }
    Sink::Replay { length } => {
      let thread = spawn_replay_writer(
        ReplayWriterConfig {
          encoder: Some(VideoEncoder::Hevc),
          fps: spec.fps,
          height: spec.height,
          keyframes,
          length,
          microphone_format: None,
          on_failure: Arc::clone(on_failure),
          primary_video: false,
          stats: Arc::clone(&stats),
          system_audio: false,
          timeline_origin: Arc::clone(timeline_origin),
          width: spec.width,
        },
        "screenwide-replay-camera-writer",
      )?;
      (None, thread)
    }
  };
  let WriterThread {
    commands,
    first_frame,
    worker,
  } = thread;
  let stream = camera::start(
    spec,
    camera_flipped,
    commands.clone(),
    Arc::clone(monitor),
    stats,
  )?;

  Ok(CameraWriterSetup {
    first_frame: Some(first_frame),
    primary_spec: None,
    secondary: Some(CameraObjects {
      commands,
      path,
      stream: Some(stream),
      worker: Some(worker),
    }),
  })
}
