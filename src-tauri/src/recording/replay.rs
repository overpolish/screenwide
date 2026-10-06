// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer: the last stretch of what the recording bar was set to
//! capture, held in memory until the user saves it as a clip.
//!
//! It runs beside the recording state machine, not inside it. A replay is
//! never "recording" as far as the bar, the dock or the tray's recording
//! controls are concerned, and a recording can start and stop while it runs.
//! Its capture settings are taken when it starts; changing the bar afterwards
//! sets up the next recording, not the replay.
//!
//! A save holds what has happened since the previous save, and never more than
//! [`REPLAY_LENGTH`]: two saves twelve seconds apart give a clip of twelve
//! seconds, not the whole buffer again. Turning the buffer off forgets where
//! the last save ended.

// Linux has no replay buffer; there the state stays off and the lifecycle
// below is never reached.
#![cfg_attr(
  not(any(target_os = "macos", target_os = "windows")),
  allow(dead_code, unused_imports)
)]

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod capture;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(in crate::recording) mod ring;
mod sleep;
pub(super) use sleep::{did_wake, will_sleep};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::StartRecordingOptions;

/// How far back a clip reaches.
pub const REPLAY_LENGTH: Duration = Duration::from_secs(30);

const REPLAY_STATE_EVENT: &str = "replay://state";
const REPLAY_ERROR_EVENT: &str = "replay://error";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReplayStatus {
  #[default]
  Off,
  Starting,
  On,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySnapshot {
  /// Whether this platform has a replay buffer at all.
  pub available: bool,
  pub length_seconds: u64,
  /// A clip is being written.
  pub saving: bool,
  pub status: ReplayStatus,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayErrorPayload {
  message: String,
}

#[derive(Default)]
pub struct ReplayState(Mutex<Inner>);

#[derive(Default)]
struct Inner {
  /// Bumped by every start and stop, so a start that finishes after the user
  /// has already turned the buffer off is discarded.
  generation: u64,
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  running: Option<Arc<capture::RunningReplay>>,
  /// What the buffer was turned on with, while it is on or starting.
  options: Option<StartRecordingOptions>,
  /// What to turn the buffer back on with once the computer wakes, if it was
  /// on when the computer went to sleep.
  resume_on_wake: Option<StartRecordingOptions>,
  saving: bool,
  status: ReplayStatus,
}

impl Inner {
  fn snapshot(&self) -> ReplaySnapshot {
    ReplaySnapshot {
      available: cfg!(any(target_os = "macos", target_os = "windows")),
      length_seconds: REPLAY_LENGTH.as_secs(),
      saving: self.saving,
      status: self.status,
    }
  }
}

fn inner(app: &AppHandle) -> std::sync::MutexGuard<'_, Inner> {
  app
    .state::<ReplayState>()
    .inner()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn snapshot(app: &AppHandle) -> ReplaySnapshot {
  inner(app).snapshot()
}

/// Mutates under the lock, then emits and refreshes the tray with the lock
/// released.
///
/// The Save Replay binding follows the status. A save, which can arrive from
/// that very shortcut's native callback, only flips `saving` and so never
/// touches the shortcut registry, which cannot be re-entered from there.
fn update<T>(app: &AppHandle, change: impl FnOnce(&mut Inner) -> T) -> T {
  let (result, before, snapshot) = {
    let mut inner = inner(app);
    let before = inner.status;
    let result = change(&mut inner);
    (result, before, inner.snapshot())
  };
  let _ = app.emit(REPLAY_STATE_EVENT, snapshot);
  #[cfg(desktop)]
  crate::tray::refresh(app);
  if before != snapshot.status {
    if let Err(error) = crate::shortcuts::sync_replay_enabled(app) {
      eprintln!("Could not update the Save Replay shortcut: {error}");
    }
  }
  result
}

/// Tells the user the buffer failed: as an event for the bar, and in an alert
/// titled `title`, since the buffer may have been started from the tray or by
/// a wake with nothing else on screen.
fn report(app: &AppHandle, title: &str, message: &str) {
  eprintln!("Replay buffer: {message}");
  let _ = app.emit(
    REPLAY_ERROR_EVENT,
    ReplayErrorPayload {
      message: message.to_owned(),
    },
  );
  crate::alert::show(app, title, message);
}

/// Starts the buffer with `options`, the recording bar's settings as they are
/// now. Returns once the start is under way; the state event says when it is
/// running or that it could not start.
pub fn start(app: &AppHandle, options: StartRecordingOptions) -> Result<(), String> {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    super::session::validate_options(&options)?;
    let generation = update(app, |inner| {
      if inner.status != ReplayStatus::Off {
        return Err("The replay buffer is already on".to_owned());
      }
      inner.status = ReplayStatus::Starting;
      inner.generation += 1;
      inner.options = Some(options.clone());
      inner.resume_on_wake = None;
      Ok(inner.generation)
    })?;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
      let started = capture::start(&app, &options);
      let leftover = update(&app, |inner| {
        if inner.generation != generation {
          // Turned off while starting: what started is not wanted.
          return started.ok();
        }
        match started {
          Ok(running) => {
            inner.running = Some(Arc::new(running));
            inner.status = ReplayStatus::On;
            None
          }
          Err(error) => {
            inner.status = ReplayStatus::Off;
            inner.options = None;
            report(&app, "Replay buffer could not turn on", &error);
            None
          }
        }
      });
      drop(leftover);
    });
    Ok(())
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (app, options);
    Err("The replay buffer is not available on this platform yet".to_owned())
  }
}

/// Turns the buffer off and lets go of everything it held.
pub fn stop(app: &AppHandle) {
  turn_off(app, false);
}

/// Turns the buffer off, keeping what it was turned on with when
/// `resume_on_wake`, so the computer waking can turn it back on.
fn turn_off(app: &AppHandle, resume_on_wake: bool) {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    let running = update(app, |inner| {
      inner.generation += 1;
      inner.status = ReplayStatus::Off;
      let options = inner.options.take();
      inner.resume_on_wake = options.filter(|_| resume_on_wake);
      inner.running.take()
    });
    // Stopping the streams and joining the writers waits on the capture
    // system, so it happens off the caller's thread.
    if let Some(running) = running {
      tauri::async_runtime::spawn_blocking(move || drop(running));
    }
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = (app, resume_on_wake);
}

/// Saves what is new since the last save, up to [`REPLAY_LENGTH`], as a
/// project of its own.
pub fn save(app: &AppHandle) -> Result<(), String> {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    // The clip ends when the user asked, not when a worker got round to it.
    let at = std::time::Instant::now();
    let running = update(app, |inner| {
      if inner.saving {
        return Err("A replay is already being saved".to_owned());
      }
      let running = inner
        .running
        .clone()
        .ok_or_else(|| "The replay buffer is not on".to_owned())?;
      inner.saving = true;
      Ok(running)
    })?;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
      if let Err(error) = capture::save(&app, &running, at) {
        report(&app, "Replay could not be saved", &error);
      }
      update(&app, |inner| inner.saving = false);
    });
    Ok(())
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = app;
    Err("The replay buffer is not available on this platform yet".to_owned())
  }
}

pub fn is_on(app: &AppHandle) -> bool {
  inner(app).status == ReplayStatus::On
}
