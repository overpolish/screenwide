// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform-neutral cursor recording stream.
//!
//! Native adapters only translate their mouse events and current cursor into
//! [`RawCursorEvent`]. Timing, pause removal, throttling and the file format
//! live here so macOS and Windows produce the same sidecar.

#[cfg(target_os = "macos")]
mod platform_macos;
#[cfg(target_os = "macos")]
use self::platform_macos as platform;
#[cfg(target_os = "windows")]
mod platform_windows;
#[cfg(target_os = "windows")]
use self::platform_windows as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::platform_unsupported as platform;

mod format;
/// What a cursor is, read from its picture where the system does not say.
#[cfg(any(target_os = "macos", test))]
mod shape;
mod visibility;
pub(crate) use visibility::set_cursor_visibility;
#[cfg(test)]
mod tests;

mod event_writer;
mod front_app;
mod own_presses;
mod rolling;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
pub(crate) use self::format::CursorSourceKind;
pub(crate) use self::format::{
  read, ButtonState, CursorButton, CursorRecord, CursorSource, CursorStyle, FORMAT_VERSION,
};
pub(crate) use self::rolling::CursorBaseline;
use crate::recording::clock::SidecarClock;
use crate::recording::sidecar_output::{self, RollingRecords, SidecarOutput};
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
  /// Another app came to the front.
  AppSwitch,
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
  output: SidecarOutput<CursorRecord, CursorBaseline>,
}

type EventSink = Arc<dyn Fn(RawCursorEvent) -> bool + Send + Sync>;

fn header(source: CursorSource) -> CursorRecord {
  CursorRecord::Header {
    coordinate_space: if cfg!(target_os = "windows") {
      "global-physical-pixels".to_owned()
    } else {
      "global-logical-points".to_owned()
    },
    platform: std::env::consts::OS.to_owned(),
    source,
    timebase: "recording-microseconds".to_owned(),
    version: FORMAT_VERSION,
  }
}

/// The native pointer tap and the stream it feeds, whatever that stream is
/// written to.
struct Tap {
  state: Arc<Mutex<StreamWriter>>,
  stop: Arc<AtomicBool>,
  worker: Option<JoinHandle<()>>,
}

impl Tap {
  /// `include_own_windows` says whether the recording shows Screenwide's own
  /// windows; where it does not, presses on them are left out.
  fn start(
    output: SidecarOutput<CursorRecord, CursorBaseline>,
    origin: Arc<OnceLock<Instant>>,
    include_own_windows: bool,
  ) -> Result<Self, String> {
    let state = Arc::new(Mutex::new(StreamWriter {
      clock: SidecarClock::new(origin),
      failure: None,
      last_appearance: None,
      last_flush: Instant::now(),
      last_move: None,
      last_visibility: None,
      last_position: None,
      output,
    }));
    let mut sink = visibility::sink(&state);
    if !include_own_windows {
      sink = own_presses::without_own_presses(sink, platform::on_own_window);
    }
    let stop = Arc::new(AtomicBool::new(false));
    let worker = platform::start(Arc::clone(&stop), sink)?;
    Ok(Self {
      state,
      stop,
      worker: Some(worker),
    })
  }

  fn state(&self) -> std::sync::MutexGuard<'_, StreamWriter> {
    self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  fn finish(&mut self) -> Result<(), String> {
    self.state().clock.stop();
    self.stop.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      worker
        .join()
        .map_err(|_| "The cursor recorder stopped unexpectedly".to_owned())?;
    }
    let mut state = self.state();
    state.output.flush()?;
    state.failure.take().map_or(Ok(()), Err)
  }
}

impl Drop for Tap {
  fn drop(&mut self) {
    let _ = self.finish();
  }
}

/// A cursor sidecar being filled beside one native recording.
pub struct CursorRecorder {
  path: PathBuf,
  tap: Tap,
}

impl CursorRecorder {
  pub fn start(
    path: PathBuf,
    origin: Arc<OnceLock<Instant>>,
    source: CursorSource,
    include_own_windows: bool,
  ) -> Result<Self, String> {
    let output = SidecarOutput::file(&path, &header(source))?;
    let tap = Tap::start(output, origin, include_own_windows).inspect_err(|_| {
      let _ = std::fs::remove_file(&path);
    })?;
    Ok(Self { path, tap })
  }

  pub fn pause(&self, at: Instant) {
    self.tap.state().clock.pause(at);
  }

  pub fn resume(&self, at: Instant) {
    self.tap.state().clock.resume(at);
  }

  pub fn stop(mut self) -> Result<PathBuf, String> {
    if let Err(error) = self.tap.finish() {
      let _ = std::fs::remove_file(&self.path);
      return Err(error);
    }
    Ok(self.path.clone())
  }

  pub fn cancel(mut self) {
    let _ = self.tap.finish();
    let _ = std::fs::remove_file(&self.path);
  }
}

/// The cursor over the last stretch of time, for the replay buffer to cut
/// clips from. It owns no file until a clip is written.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub struct RollingCursorRecorder {
  header: CursorRecord,
  tap: Tap,
}

#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
impl RollingCursorRecorder {
  pub fn start(
    origin: Arc<OnceLock<Instant>>,
    source: CursorSource,
    include_own_windows: bool,
    horizon: Duration,
  ) -> Result<Self, String> {
    let horizon_us = u64::try_from(horizon.as_micros()).unwrap_or(u64::MAX);
    let output = SidecarOutput::Rolling(RollingRecords::new(horizon_us));
    Ok(Self {
      header: header(source),
      tap: Tap::start(output, origin, include_own_windows)?,
    })
  }

  /// Writes the sidecar for the clip `start_us..=end_us` of recording time.
  pub fn write_clip(&self, path: &Path, start_us: u64, end_us: u64) -> Result<(), String> {
    let records = {
      let state = self.tap.state();
      if let Some(failure) = &state.failure {
        return Err(failure.clone());
      }
      state
        .output
        .rolling()
        .expect("a rolling recorder writes to a rolling output")
        .clip(start_us, end_us)
    };
    sidecar_output::write_file(path, &self.header, &records)
  }
}
