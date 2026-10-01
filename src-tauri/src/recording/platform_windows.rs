// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows screen recording: Windows Graphics Capture into Media Foundation.
//!
//! WGC hands D3D11 textures to a bounded channel. The capture callback never
//! waits for the encoder, and Media Foundation consumes those textures through
//! its DXGI device manager without a GPU-to-CPU copy or an FFmpeg subprocess.

mod audio;
mod camera;
mod capture;
mod desktop_compositor;
mod writer;

mod audio_clock;
mod session;
mod source_plan;
mod startup;
mod startup_support;
#[cfg(test)]
mod tests;
use source_plan::{resolve_source, ResolvedSource};
pub use startup::begin_blocking;
use startup_support::{begin_audio_only, both_first_frames, spawn_writer};

use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc, Arc, Mutex, OnceLock,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use capture::{CaptureObjects, CaptureTarget};
use desktop_compositor::DesktopFrameCoordinator;
use writer::{Command, WriterConfig};

use crate::{
  capture_geometry::{physical_capture_rect, video_capture_rect, CaptureRect},
  desktop_capture::{self, CapturePlan, DesktopDisplay, OutputLimits},
};

use super::encoding::FinalizeInfo;
use super::{
  cursor::{CursorSource, CursorSourceKind},
  CaptureStartupConfig, PrimaryCaptureSource,
};

const FINALIZE_TIMEOUT: Duration = Duration::from_secs(30);

pub struct CaptureStart {
  pub cursor_source: Option<super::cursor::CursorSource>,
  pub first_frame: mpsc::Receiver<Result<(), String>>,
  pub session: CaptureSession,
  pub source_scale_factor: f32,
  pub timeline_origin: Arc<OnceLock<Instant>>,
}

pub struct CaptureSession {
  audio: Option<audio::AudioCaptures>,
  audio_only_clock: Option<AudioOnlyClock>,
  audio_only_path: Option<std::path::PathBuf>,
  camera: Option<CameraRecording>,
  captures: Vec<CaptureObjects>,
  commands: Option<mpsc::SyncSender<Command>>,
  primary_camera: Option<camera::CameraStream>,
  stopped_at: Arc<OnceLock<Instant>>,
  worker: Option<JoinHandle<()>>,
}

struct CameraRecording {
  commands: mpsc::SyncSender<Command>,
  path: std::path::PathBuf,
  stream: Option<camera::CameraStream>,
  worker: Option<JoinHandle<()>>,
}

struct AudioOnlyClock {
  paused: Mutex<(Option<Instant>, Duration)>,
  started: Instant,
}

type WriterSpawn = (
  mpsc::SyncSender<Command>,
  mpsc::Receiver<Result<(), String>>,
  JoinHandle<()>,
);
