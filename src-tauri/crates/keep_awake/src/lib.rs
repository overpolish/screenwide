// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps the computer from sleeping when idle, so work left running carries
//! on while nobody is at it, and on request keeps the display on as well.
//!
//! Both are power assertions: nothing runs while they are held and no input
//! is sent, so chat apps still show the user away. [`KeepAwake`] holds off
//! idle sleep alone: the display still turns off and the screen still locks
//! on the user's timers. [`ScreenOn`] holds the display on too, except while
//! the screen is locked. Closing a laptop's lid still sleeps it.

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

mod screen_on;

pub use screen_on::ScreenOn;

/// Keeps the computer awake until dropped.
pub struct KeepAwake {
  _hold: platform::Hold,
}

impl KeepAwake {
  /// `reason` is what the system lists as keeping it awake, in
  /// `pmset -g assertions` or `powercfg /requests`.
  pub fn start(reason: &str) -> Result<Self, String> {
    platform::Hold::system(reason).map(|hold| Self { _hold: hold })
  }
}
