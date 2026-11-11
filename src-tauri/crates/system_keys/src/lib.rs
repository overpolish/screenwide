// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's own screenshot keys, taken for Screenwide's tools while a
//! [`SystemKeys`] is held. Neither platform lets an app register them: on
//! Windows the shell owns Win+Shift+S and its neighbours, and on macOS the
//! screenshot keys are the system's own hot keys. So each press is seen
//! before the system sees it, by a low-level keyboard hook on Windows and an
//! event tap on macOS, which keeps the ones Screenwide takes. Both go with the
//! process, and the system's hot key settings are never changed, so nothing
//! has to be given back after a crash.

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
  /// Cmd+Shift+4: a picture of an area, saved.
  AreaScreenshot,
  /// Cmd+Ctrl+Shift+4: a picture of an area, copied.
  AreaScreenshotToClipboard,
  /// Cmd+Shift+5: the Screenshot app's toolbar.
  ScreenshotToolbar,
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
