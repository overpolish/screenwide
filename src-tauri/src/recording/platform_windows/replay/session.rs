// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A running replay buffer, as the shared replay lifecycle sees it.

use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::super::writer::{Command, MediaFoundation};
use super::super::{audio, startup, CaptureSession};
use super::{movie, ClipFrom, ClipRequest, ClipTake};
use crate::recording::cursor::CursorSource;
use crate::recording::CaptureStartupConfig;

/// How long a writer may take to hand over its stretch. It only flushes the
/// encoder and copies references, so this is only ever reached by a writer
/// that has stopped answering.
const TAKE_TIMEOUT: Duration = Duration::from_secs(10);
/// A save shorter than this has nothing new worth opening.
const MIN_CLIP_NS: i64 = 100_000_000;
const NANOS_PER_MS: i64 = 1_000_000;

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
  timeline_origin: Arc<OnceLock<Instant>>,
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
pub fn begin_replay_blocking(
  config: CaptureStartupConfig,
  length: Duration,
) -> Result<ReplayCapture, String> {
  let start = startup::begin(config, Some(length))?;
  Ok(ReplayCapture {
    cursor_source: start.cursor_source,
    first_frame: start.first_frame,
    session: ReplaySession {
      capture: start.session,
      length,
      timeline_origin: Arc::clone(&start.timeline_origin),
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

fn span_ms(start_ns: i64, end_ns: i64) -> u64 {
  u64::try_from(end_ns.saturating_sub(start_ns) / NANOS_PER_MS).unwrap_or_default()
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
    let latest = ClipFrom::Latest {
      length_ns,
      since_ns,
    };
    let primary = match &self.capture.commands {
      Some(commands) => take(commands, at, latest)?,
      None => self.audio_only(at, latest)?,
    };
    let camera = self.capture.camera.as_ref().and_then(|camera| {
      let from = ClipFrom::Follow {
        start_ns: primary.start_ns,
      };
      // The screen is the clip; a camera with nothing to give is left out
      // rather than costing the save.
      take(&camera.commands, at, from)
        .inspect_err(|error| eprintln!("The replay's camera has no clip: {error}"))
        .ok()
    });
    if primary.end_ns - primary.start_ns < MIN_CLIP_NS {
      return Err("Nothing new has happened since the last replay was saved".to_owned());
    }

    let _media_foundation = MediaFoundation::start()?;
    let duration_ms = span_ms(primary.start_ns, primary.end_ns);
    if let Some(video) = &primary.video {
      movie::write_video(path, video, primary.start_ns, primary.end_ns)?;
    }
    let (has_microphone, has_system_audio) = self
      .write_audio(path, &primary, duration_ms)
      .inspect_err(|_| {
        let _ = std::fs::remove_file(path);
      })?;
    let camera = camera.and_then(|take| {
      let video = take.video.as_ref()?;
      match movie::write_video(camera_path, video, take.start_ns, take.end_ns) {
        Ok(()) => Some(SavedCamera {
          duration_ms: span_ms(take.start_ns, take.end_ns),
          height: video.height,
          width: video.width,
        }),
        Err(error) => {
          eprintln!("The replay's camera could not be saved: {error}");
          None
        }
      }
    });
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

  /// An audio-only buffer has no writer to ask: its clip is simply the
  /// stretch of wall time the save asks for.
  fn audio_only(&self, at: Instant, from: ClipFrom) -> Result<ClipTake, String> {
    let ClipFrom::Latest {
      length_ns,
      since_ns,
    } = from
    else {
      return Err("An audio-only replay chooses its own clip".to_owned());
    };
    let origin = self
      .timeline_origin
      .get()
      .copied()
      .ok_or_else(|| "The replay buffer has not captured anything yet".to_owned())?;
    let end_ns = i64::try_from(at.saturating_duration_since(origin).as_nanos()).unwrap_or(i64::MAX);
    let start_ns = end_ns
      .saturating_sub(length_ns)
      .max(since_ns.unwrap_or(0))
      .max(0);
    Ok(ClipTake {
      start_ns,
      end_ns,
      video: None,
    })
  }

  /// Adds the clip's audio to the movie at `path`, or writes it there on its
  /// own, as a recording's audio is muxed when it stops.
  fn write_audio(
    &self,
    path: &Path,
    clip: &ClipTake,
    duration_ms: u64,
  ) -> Result<(bool, bool), String> {
    let Some(captures) = &self.capture.audio else {
      return Ok((false, false));
    };
    let files = captures.clip(path, clip.start_ns, clip.end_ns)?;
    let written = (files.has_microphone, files.has_system_audio);
    let sidecars = files.paths();
    let muxed = if clip.video.is_some() {
      audio::mux(path, duration_ms, files)
    } else {
      audio::mux_audio_only(path, duration_ms, files)
    };
    // A mux removes the sidecars only once it has succeeded.
    if muxed.is_err() {
      for sidecar in sidecars {
        let _ = std::fs::remove_file(sidecar);
      }
    }
    muxed.map(|()| written)
  }
}
