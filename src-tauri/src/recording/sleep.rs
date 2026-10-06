// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What capture does when the computer sleeps.
//!
//! A recording left running is paused the moment the computer goes to sleep,
//! as if the user had pressed Pause, so it never carries on unattended after
//! a wake. It stays paused until the user resumes or stops it. A recording
//! still counting down to its start is discarded instead: one that starts on
//! its own after a wake is never what was asked for. The replay buffer turns
//! off with the computer and back on once it wakes.
//!
//! Each platform only reports the two moments; what they mean is decided
//! here, the same way everywhere.

#[cfg(target_os = "macos")]
mod platform_macos;
#[cfg(target_os = "macos")]
use self::platform_macos as platform;
#[cfg(target_os = "windows")]
mod platform_windows;
#[cfg(target_os = "windows")]
use self::platform_windows as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_other;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::platform_other as platform;

use tauri::AppHandle;

use super::RecordingStatus;

/// Installs the platform's sleep and wake observer once, at startup.
pub(crate) fn initialize(app: &AppHandle) {
  platform::initialize(app, will_sleep, did_wake);
}

fn will_sleep(app: &AppHandle) {
  let result = match super::snapshot(app).status {
    RecordingStatus::Recording => super::pause(app),
    RecordingStatus::Starting => super::cancel(app),
    _ => Ok(()),
  };
  if let Err(error) = result {
    eprintln!("The recording could not be set aside for sleep: {error}");
  }
  super::replay::will_sleep(app);
}

fn did_wake(app: &AppHandle) {
  super::replay::did_wake(app);
}
