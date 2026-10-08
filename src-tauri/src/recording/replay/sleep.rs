// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer across the computer sleeping: off while it sleeps, and
//! back on with the same settings once it wakes.
//!
//! Turning off rather than pausing lets go of the camera, the microphone and
//! the memory while nothing is happening, and a buffer started fresh after a
//! wake does not depend on capture, encoders or audio devices surviving the
//! sleep. What it held was the moment the computer went to sleep.

use std::time::Duration;

use tauri::AppHandle;

use super::{inner, report, ReplayStatus};

/// How long after a wake the buffer waits before starting again. Displays,
/// cameras and audio devices come back a little after the system does, and a
/// start that finds them missing fails.
const WAKE_SETTLE: Duration = Duration::from_secs(3);

pub(in crate::recording) fn will_sleep(app: &AppHandle) {
  super::turn_off(app, true);
}

pub(in crate::recording) fn did_wake(app: &AppHandle) {
  let (options, generation) = {
    let mut inner = inner(app);
    let Some(options) = inner.resume_on_wake.take() else {
      return;
    };
    (options, inner.generation)
  };
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(WAKE_SETTLE);
    // The user turned the buffer on or off in the meantime: theirs is the
    // later word.
    {
      let inner = inner(&app);
      if inner.generation != generation || inner.status != ReplayStatus::Off {
        return;
      }
    }
    if let Err(error) = super::start(&app, options) {
      report(
        &app,
        &crate::i18n::t!("alert-replay-restart-failed"),
        &error,
      );
    }
  });
}
