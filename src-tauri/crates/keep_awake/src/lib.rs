// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake: the computer stays awake, and its user shown as present in
//! chat apps, while they are away from it, with nothing they would notice.
//!
//! Chat apps mark someone away from how long the system has gone without
//! input: on macOS the combined session's time since the last input event,
//! which Chromium reads, and on Windows `GetLastInputInfo`. Power management
//! dims, sleeps and locks on the same count. Power assertions, such as
//! `caffeinate` or `SetThreadExecutionState`, hold off sleep but leave that
//! count running, so it is reset the only way it can be: by an input event.
//! Once the user has been idle for [`NUDGE_AFTER`], one mouse move to where
//! the pointer already is goes out, and the cursor stays put. Each carries
//! [`EVENT_TAG`], so Screenwide's own event taps can leave it out of a
//! recording. On Windows the move also restarts the power timers, since
//! Windows drops input sent while an elevated app is in front.
//!
//! Nothing is sent while the screen is locked or the display asleep: locking
//! stays the way to be shown away, and a sleeping display is not woken.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use unsupported as platform;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

/// Marks the input events Keep Awake sends: macOS's event source user data,
/// Windows' extra info. Spells "SWKA".
pub const EVENT_TAG: i64 = 0x5357_4b41;

/// Under a minute, the shortest time either system offers before the display
/// goes off, so a move always comes before it.
const NUDGE_AFTER: Duration = Duration::from_secs(50);

/// Whether Keep Awake may send input. macOS asks for Accessibility; Windows
/// needs nothing.
pub fn permitted() -> bool {
  platform::permitted()
}

/// Keeps the computer awake until dropped.
pub struct KeepAwake {
  stop: Arc<(Mutex<bool>, Condvar)>,
  worker: Option<JoinHandle<()>>,
}

impl KeepAwake {
  pub fn start() -> Result<Self, String> {
    let stop = Arc::new((Mutex::new(false), Condvar::new()));
    let signal = Arc::clone(&stop);
    let worker = std::thread::Builder::new()
      .name("screenwide-keep-awake".to_owned())
      .spawn(move || run(&signal))
      .map_err(|error| format!("Could not start Keep Awake: {error}"))?;
    Ok(Self {
      stop,
      worker: Some(worker),
    })
  }
}

impl Drop for KeepAwake {
  fn drop(&mut self) {
    let (stopped, wake) = &*self.stop;
    *stopped.lock().unwrap_or_else(PoisonError::into_inner) = true;
    wake.notify_one();
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
  }
}

/// Sleeps until the user has been idle for [`NUDGE_AFTER`], nudges, and
/// sleeps again: one wake-up a cycle, none while the user is busy beyond the
/// one that finds them so.
fn run(stop: &(Mutex<bool>, Condvar)) {
  let (stopped, wake) = stop;
  let mut guard = stopped.lock().unwrap_or_else(PoisonError::into_inner);
  while !*guard {
    let idle = platform::idle();
    let wait = match NUDGE_AFTER.checked_sub(idle) {
      Some(left) if !left.is_zero() => left,
      _ => {
        if platform::screen_in_use() {
          platform::nudge();
        }
        NUDGE_AFTER
      }
    };
    guard = wake
      .wait_timeout_while(guard, wait, |stopped| !*stopped)
      .unwrap_or_else(PoisonError::into_inner)
      .0;
  }
}
