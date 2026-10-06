// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A privacy-filtered keyboard shortcut sidecar.
//!
//! Native adapters report physical key presses and focused-control context.
//! This shared layer owns acceptance, recording time, pause removal and the
//! versioned file format. It never receives or persists typed characters:
//! typing into a field is kept only as when it happened, twice a second at
//! most, for the cursor to hide and a zoom to hold on.

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
mod rolling;
#[cfg(test)]
mod tests;
mod writer;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(crate) use format::{read, typing_times_us, KeyboardModifier, KeyboardRecord, FORMAT_VERSION};
#[cfg(test)]
use writer::modifier_transition_is_down;
use writer::StreamWriter;

pub(crate) use self::rolling::KeyboardBaseline;
use crate::recording::clock::SidecarClock;
use crate::recording::sidecar_output::{self, RollingRecords, SidecarOutput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FocusContext {
  NonText,
  Secure,
  Text,
  Unknown,
}

impl FocusContext {
  /// Focus can change between the event-tap callback and the Accessibility
  /// query that follows it. Accept a lone printable key only when both sides
  /// agree it was outside text; every disagreement fails closed.
  fn conservative(before: Self, after: Self) -> Self {
    if before == Self::Secure || after == Self::Secure {
      Self::Secure
    } else if before == Self::Text || after == Self::Text {
      Self::Text
    } else if before == Self::NonText && after == Self::NonText {
      Self::NonText
    } else {
      Self::Unknown
    }
  }
}

#[derive(Clone, Debug)]
pub(super) struct RawKeyboardEvent {
  pub at: Instant,
  pub focus: FocusContext,
  pub kind: RawKeyboardEventKind,
  pub key_code: u16,
  pub modifiers: Vec<KeyboardModifier>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RawKeyboardEventKind {
  KeyDown {
    is_printable: bool,
    is_repeat: bool,
  },
  KeyUp,
  FlagsChanged {
    is_down: bool,
    modifier: KeyboardModifier,
  },
}

type EventSink = Arc<dyn Fn(RawKeyboardEvent) -> bool + Send + Sync>;

fn header() -> KeyboardRecord {
  KeyboardRecord::Header {
    platform: std::env::consts::OS.to_owned(),
    timebase: "recording-microseconds".to_owned(),
    version: FORMAT_VERSION,
  }
}

/// The native key tap and the stream it feeds, whatever that stream is
/// written to.
struct Tap {
  state: Arc<Mutex<StreamWriter>>,
  stop: Arc<AtomicBool>,
  worker: Option<JoinHandle<()>>,
}

impl Tap {
  fn start(
    output: SidecarOutput<KeyboardRecord, KeyboardBaseline>,
    origin: Arc<OnceLock<Instant>>,
  ) -> Result<Self, String> {
    let state = Arc::new(Mutex::new(StreamWriter {
      active_keys: HashSet::new(),
      clock: SidecarClock::new(origin),
      failure: None,
      output,
      last_typing_us: None,
    }));
    let sink_state = Arc::clone(&state);
    let sink: EventSink = Arc::new(move |event| {
      let mut state = sink_state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      if state.failure.is_some() {
        return false;
      }
      match state.record(event) {
        Ok(recorded) => recorded,
        Err(error) => {
          eprintln!("Keyboard shortcut recording stopped writing: {error}");
          state.failure = Some(error);
          false
        }
      }
    });
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
        .map_err(|_| "The keyboard shortcut recorder stopped unexpectedly".to_owned())?;
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

pub struct KeyboardRecorder {
  path: PathBuf,
  tap: Tap,
}

impl KeyboardRecorder {
  pub fn start(path: PathBuf, origin: Arc<OnceLock<Instant>>) -> Result<Self, String> {
    let output = SidecarOutput::file(&path, &header())?;
    let tap = Tap::start(output, origin).inspect_err(|_| {
      let _ = std::fs::remove_file(&path);
    })?;
    Ok(Self { path, tap })
  }

  pub fn pause(&self, at: Instant) {
    let mut state = self.tap.state();
    // A key may be released while recording time is frozen. Forget the live
    // set at the boundary so that release cannot leave a stale accepted key
    // suppressing the first press after resume.
    state.active_keys.clear();
    state.clock.pause(at);
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

/// Keyboard shortcuts over the last stretch of time, for the replay buffer to
/// cut clips from. It owns no file until a clip is written.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct RollingKeyboardRecorder {
  tap: Tap,
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
impl RollingKeyboardRecorder {
  pub fn start(origin: Arc<OnceLock<Instant>>, horizon: Duration) -> Result<Self, String> {
    let horizon_us = u64::try_from(horizon.as_micros()).unwrap_or(u64::MAX);
    let output = SidecarOutput::Rolling(RollingRecords::new(horizon_us));
    Ok(Self {
      tap: Tap::start(output, origin)?,
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
    sidecar_output::write_file(path, &header(), &records)
  }
}
