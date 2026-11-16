// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The app a dictation pastes into: the one in front as listening started,
//! then whichever other app the person brings in front while dictating, so
//! the text goes where they last were. Screenwide never takes its place: a
//! press on the island can bring Screenwide in front, and the paste then
//! brings back the app that was in front before it.

use std::time::{Duration, Instant};

use screenwide_paste::Target;
use tauri::AppHandle;

use super::with;

/// How often the app in front is looked at while listening.
const LOOK_EVERY: Duration = Duration::from_millis(250);

/// What a dictation knows of the app in front.
pub(super) struct Front {
  target: Option<Target>,
  looked: Instant,
}

impl Front {
  /// The app in front as listening starts.
  pub fn now() -> Self {
    Self {
      target: Target::current(),
      looked: Instant::now(),
    }
  }

  /// The app to paste into.
  pub fn target(&self) -> Option<Target> {
    self.target
  }
}

/// Looks at the app in front, unless it was looked at lately. Called as the
/// microphone's level comes in, a few times a second; macOS answers which
/// app is in front on its main thread.
pub(super) fn follow(app: &AppHandle) {
  let due = with(|dictation| {
    dictation.listening.as_mut().is_some_and(|listening| {
      let due = listening.front.looked.elapsed() >= LOOK_EVERY;
      if due {
        listening.front.looked = Instant::now();
      }
      due
    })
  });
  if due {
    let _ = app.run_on_main_thread(look);
  }
}

/// Takes the app in front as the one to paste into, unless it is
/// Screenwide.
pub(super) fn look() {
  let Some(target) = Target::current() else {
    return;
  };
  with(|dictation| {
    if let Some(listening) = dictation.listening.as_mut() {
      listening.front.target = Some(target);
    }
  });
}
