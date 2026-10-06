// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A running replay buffer, as the shared replay lifecycle sees it.

use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::super::output::Command;
use super::super::session::CaptureSession;
use super::super::startup::{self, Sink};
use super::mux::{write_movie, WrittenMovie};
use super::{ClipFrom, ClipRequest, ClipTake};
use crate::recording::cursor::CursorSource;
use crate::recording::CaptureStartupConfig;

/// How long a writer may take to hand over its stretch. It only flushes the
/// encoder and copies references, so this is only ever reached by a writer
/// that has stopped answering.
const TAKE_TIMEOUT: Duration = Duration::from_secs(10);
/// A save shorter than this has nothing new worth opening.
const MIN_CLIP_NS: i64 = 100_000_000;

pub struct ReplayCapture {
  pub cursor_source: Option<CursorSource>,
  pub first_frame: Receiver<Result<(), String>>,
  pub session: ReplaySession,
  pub source_scale_factor: f32,
  pub timeline_origin: Arc<OnceLock<Instant>>,
}

pub struct ReplaySession {
  capture: CaptureSession,
  length: Duration,
}

/// A clip written to disk, in the replay buffer's own time.
pub struct SavedClip {
  pub camera: Option<SavedCamera>,
  pub duration_ms: u64,
  pub end_ns: i64,
  pub has_microphone: bool,
  pub has_system_audio: bool,
  pub height: u32,
  pub start_ns: i64,
  pub width: u32,
}

pub struct SavedCamera {
  pub duration_ms: u64,
  pub height: u32,
  pub width: u32,
}

/// Opens the capture `config` describes with its media held in memory.
/// `config.path` and `config.camera_path` are not used: clips are written
/// wherever [`ReplaySession::save`] is told.
pub fn begin_blocking(
  config: CaptureStartupConfig,
  length: Duration,
) -> Result<ReplayCapture, String> {
  let start = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .map_err(|error| error.to_string())?
    .block_on(startup::begin(config, Sink::Replay { length }))?;
  Ok(ReplayCapture {
    cursor_source: start.cursor_source,
    first_frame: start.first_frame,
    session: ReplaySession {
      capture: start.session,
      length,
    },
    source_scale_factor: start.source_scale_factor,
    timeline_origin: start.timeline_origin,
  })
}

fn take(commands: &SyncSender<Command>, at: Instant, from: ClipFrom) -> Result<ClipTake, String> {
  let (reply, replies) = mpsc::channel();
  commands
    .send(Command::Replay(ClipRequest { at, from, reply }))
    .map_err(|_| "The replay buffer is no longer running".to_owned())?;
  replies
    .recv_timeout(TAKE_TIMEOUT)
    .map_err(|_| "The replay buffer did not answer in time".to_owned())?
}

impl ReplaySession {
  /// Writes the clip that ends at `at` and starts no earlier than `since_ns`,
  /// the end of the previous save. The primary movie goes to `path`, a camera
  /// beside it to `camera_path`.
  pub fn save(
    &self,
    at: Instant,
    since_ns: Option<i64>,
    path: &Path,
    camera_path: &Path,
  ) -> Result<SavedClip, String> {
    let length_ns = i64::try_from(self.length.as_nanos()).unwrap_or(i64::MAX);
    let primary = take(
      &self.capture.commands,
      at,
      ClipFrom::Latest {
        length_ns,
        since_ns,
      },
    )?;
    let camera = match &self.capture.camera {
      Some(camera) => Some(take(
        &camera.commands,
        at,
        ClipFrom::Follow {
          start_ns: primary.start_ns,
        },
      )?),
      None => None,
    };
    if primary.end_ns - primary.start_ns < MIN_CLIP_NS {
      return Err("Nothing new has happened since the last replay was saved".to_owned());
    }

    let WrittenMovie {
      duration_ms,
      has_microphone,
      has_system_audio,
    } = write_movie(path, &primary)?;
    let camera = match camera.filter(|take| take.video.is_some()) {
      Some(take) => match write_movie(camera_path, &take) {
        Ok(movie) => {
          let video = take.video.as_ref().expect("filtered above");
          Some(SavedCamera {
            duration_ms: movie.duration_ms,
            height: video.height,
            width: video.width,
          })
        }
        // The screen is the clip; a camera that could not be written is
        // left out rather than costing the save.
        Err(error) => {
          eprintln!("The replay's camera could not be saved: {error}");
          None
        }
      },
      None => None,
    };
    let (width, height) = primary
      .video
      .as_ref()
      .map_or((0, 0), |video| (video.width, video.height));
    Ok(SavedClip {
      camera,
      duration_ms,
      end_ns: primary.end_ns,
      has_microphone,
      has_system_audio,
      height,
      start_ns: primary.start_ns,
      width,
    })
  }
}
