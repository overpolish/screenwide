// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform-neutral cursor recording stream.
//!
//! Native adapters only translate their mouse events and current cursor into
//! [`RawCursorEvent`]. Timing, pause removal, throttling and the file format
//! live here so macOS and Windows produce the same sidecar.

#[cfg(target_os = "macos")]
#[path = "cursor/platform_macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "cursor/platform_windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "cursor/platform_unsupported.rs"]
mod platform;

mod format;
mod visibility;
pub(crate) use visibility::set_cursor_visibility;
#[cfg(test)]
mod tests;

#[path = "cursor/event_writer.rs"]
mod event_writer;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
pub(crate) use self::format::CursorSourceKind;
pub(crate) use self::format::{
  read, ButtonState, CursorButton, CursorRecord, CursorSource, CursorStyle, FORMAT_VERSION,
};
use crate::recording::clock::SidecarClock;
const MOVEMENT_INTERVAL: Duration = Duration::from_micros(7_500);
const FLUSH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CursorAppearance {
  pub height: f64,
  pub hotspot_x: f64,
  pub hotspot_y: f64,
  pub style: CursorStyle,
  pub width: f64,
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
pub(super) enum RawCursorEventKind {
  Appearance,
  Move,
  Snapshot,
  Button {
    button: CursorButton,
    click_count: u8,
    state: ButtonState,
  },
}

#[derive(Clone, Debug)]
pub(super) struct RawCursorEvent {
  pub appearance: CursorAppearance,
  pub at: Instant,
  pub kind: RawCursorEventKind,
  pub x: f64,
  pub y: f64,
}

struct StreamWriter {
  clock: SidecarClock,
  failure: Option<String>,
  last_appearance: Option<CursorAppearance>,
  last_flush: Instant,
  last_move: Option<Instant>,
  last_visibility: Option<bool>,
  last_position: Option<(f64, f64)>,
  writer: BufWriter<File>,
}

type EventSink = Arc<dyn Fn(RawCursorEvent) -> bool + Send + Sync>;

/// A cursor sidecar being filled beside one native recording.
pub struct CursorRecorder {
  path: PathBuf,
  state: Arc<Mutex<StreamWriter>>,
  stop: Arc<AtomicBool>,
  worker: Option<JoinHandle<()>>,
}

impl CursorRecorder {
  pub fn start(
    path: PathBuf,
    origin: Arc<OnceLock<Instant>>,
    source: CursorSource,
  ) -> Result<Self, String> {
    let file = File::create(&path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(
      &mut writer,
      &CursorRecord::Header {
        coordinate_space: if cfg!(target_os = "windows") {
          "global-physical-pixels".to_owned()
        } else {
          "global-logical-points".to_owned()
        },
        platform: std::env::consts::OS.to_owned(),
        source,
        timebase: "recording-microseconds".to_owned(),
        version: FORMAT_VERSION,
      },
    )
    .map_err(|error| error.to_string())?;
    writer.write_all(b"\n").map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())?;

    let state = Arc::new(Mutex::new(StreamWriter {
      clock: SidecarClock::new(origin),
      failure: None,
      last_appearance: None,
      last_flush: Instant::now(),
      last_move: None,
      last_visibility: None,
      last_position: None,
      writer,
    }));
    let sink = visibility::sink(&state);
    let stop = Arc::new(AtomicBool::new(false));
    let worker = platform::start(Arc::clone(&stop), sink).inspect_err(|_| {
      let _ = std::fs::remove_file(&path);
    })?;

    Ok(Self {
      path,
      state,
      stop,
      worker: Some(worker),
    })
  }

  pub fn pause(&self, at: Instant) {
    self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clock
      .pause(at);
  }

  pub fn resume(&self, at: Instant) {
    self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clock
      .resume(at);
  }

  pub fn stop(mut self) -> Result<PathBuf, String> {
    if let Err(error) = self.finish() {
      let _ = std::fs::remove_file(&self.path);
      return Err(error);
    }
    Ok(self.path.clone())
  }

  pub fn cancel(mut self) {
    let _ = self.finish();
    let _ = std::fs::remove_file(&self.path);
  }

  fn finish(&mut self) -> Result<(), String> {
    {
      let mut state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      state.clock.stop();
    }
    self.stop.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      worker
        .join()
        .map_err(|_| "The cursor recorder stopped unexpectedly".to_owned())?;
    }
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.writer.flush().map_err(|error| error.to_string())?;
    state.failure.take().map_or(Ok(()), Err)
  }
}

impl Drop for CursorRecorder {
  fn drop(&mut self) {
    let _ = self.finish();
  }
}
