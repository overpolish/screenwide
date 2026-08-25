// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A privacy-filtered keyboard shortcut sidecar.
//!
//! Native adapters report physical key presses and focused-control context.
//! This shared layer owns acceptance, recording time, pause removal and the
//! versioned file format. It never receives or persists typed characters.

#[cfg(target_os = "macos")]
#[path = "keyboard/platform_macos.rs"]
mod platform;
#[cfg(not(target_os = "macos"))]
#[path = "keyboard/platform_unsupported.rs"]
mod platform;

mod format;
#[cfg(test)]
mod tests;
mod writer;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;
use std::{
  collections::HashSet,
  fs::File,
  io::{BufWriter, Write},
};

pub(crate) use format::{read, KeyboardModifier, KeyboardRecord, FORMAT_VERSION};
#[cfg(test)]
use writer::modifier_transition_is_down;
use writer::{KeyboardClock, StreamWriter};

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

pub struct KeyboardRecorder {
  path: PathBuf,
  state: Arc<Mutex<StreamWriter>>,
  stop: Arc<AtomicBool>,
  worker: Option<JoinHandle<()>>,
}

impl KeyboardRecorder {
  pub fn start(path: PathBuf, origin: Arc<OnceLock<Instant>>) -> Result<Self, String> {
    let file = File::create(&path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(
      &mut writer,
      &KeyboardRecord::Header {
        platform: std::env::consts::OS.to_owned(),
        timebase: "recording-microseconds".to_owned(),
        version: FORMAT_VERSION,
      },
    )
    .map_err(|error| error.to_string())?;
    writer.write_all(b"\n").map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())?;

    let state = Arc::new(Mutex::new(StreamWriter {
      active_keys: HashSet::new(),
      clock: KeyboardClock::new(origin),
      failure: None,
      writer,
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
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    // A key may be released while recording time is frozen. Forget the live
    // set at the boundary so that release cannot leave a stale accepted key
    // suppressing the first press after resume.
    state.active_keys.clear();
    state.clock.pause(at);
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
    self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clock
      .stop();
    self.stop.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      worker
        .join()
        .map_err(|_| "The keyboard shortcut recorder stopped unexpectedly".to_owned())?;
    }
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.writer.flush().map_err(|error| error.to_string())?;
    state.failure.take().map_or(Ok(()), Err)
  }
}

impl Drop for KeyboardRecorder {
  fn drop(&mut self) {
    let _ = self.finish();
  }
}
