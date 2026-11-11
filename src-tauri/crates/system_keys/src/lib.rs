// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's own screenshot keys, taken for Screenwide's tools while a
//! [`SystemKeys`] is held. On Windows the shell owns Win+Shift+S and its
//! neighbours, so no registered hot key can have them; a low-level keyboard
//! hook sees each press before the shell does and keeps the ones Screenwide
//! takes from it. The hook goes with the process, so nothing has to be given
//! back after a crash.

#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(not(target_os = "windows"))]
use unsupported as platform;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

/// A key the system answers with a tool of its own.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemKey {
  /// Win+Shift+S: Snipping Tool's area capture.
  ScreenClip,
  /// Print Screen alone, which Windows 11 gives to Snipping Tool.
  PrintScreen,
  /// Win+Shift+R: Snipping Tool's screen recording.
  ScreenRecord,
  /// Win+Shift+T: PowerToys' Text Extractor.
  TextExtractor,
}

/// Whether the system's keys can be taken here.
pub fn supported() -> bool {
  platform::SUPPORTED
}

/// Takes the system's keys until dropped.
pub struct SystemKeys {
  _hook: platform::Hook,
}

impl SystemKeys {
  /// Starts taking keys. `on_key` runs for each press of one, on a thread
  /// that holds up every keystroke in the system until it returns, so it
  /// must decide at once: `true` takes the key for Screenwide, `false` leaves
  /// it to the system.
  pub fn start(on_key: impl Fn(SystemKey) -> bool + Send + 'static) -> Result<Self, String> {
    platform::Hook::start(Box::new(on_key)).map(|hook| Self { _hook: hook })
  }
}

type OnKey = Box<dyn Fn(SystemKey) -> bool + Send>;
