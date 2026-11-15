// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text put where the caret is in whichever app is in front, the way a person
//! would: on the clipboard, then the paste keys. Typing it key by key is slow
//! and garbles characters in some Windows apps; pasting is what every app
//! takes.
//!
//! What the clipboard held is put back afterwards, once the app has read the
//! text: the text is a promise the system asks for when it is read, which is
//! the receipt. A fixed wait would lose the race in a busy app, which would
//! then paste the old clipboard. It is never put back over something copied in
//! the meantime. The text is marked so clipboard history and clipboard
//! managers skip it.

use std::time::{Duration, Instant};

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

/// Why the text was not pasted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
  /// A password field has the keyboard, and nothing typed for it may pass
  /// through the clipboard.
  SecureField,
  /// The app in front runs as administrator, and Windows lets nothing less
  /// privileged send it keys.
  Elevated,
  /// The clipboard or the keys could not be used; the words are English, for
  /// the log.
  Failed(String),
}

/// How long after the last time the text was read it stays on the clipboard:
/// some apps read it more than once for a paste, Chromium's first to look.
const QUIET: Duration = Duration::from_millis(200);

/// The longest the text is left on the clipboard waiting to be read, so a
/// paste the app ignored does not keep the clipboard for long.
const GIVE_UP: Duration = Duration::from_secs(8);

/// Runs work on the app's main thread, which macOS's clipboard needs.
pub type OnMain<'a> = &'a dyn Fn(Box<dyn FnOnce() + Send>);

/// Pastes `text` into the app in front, then puts the clipboard back. Blocks
/// until the clipboard is back, so it must not run on the main thread; work
/// that needs that thread is handed to `on_main`.
pub fn paste(text: &str, on_main: OnMain<'_>) -> Result<(), Refusal> {
  platform::paste(text, on_main)
}

/// Whether the clipboard can be put back, from when the paste keys were sent
/// and when the text was last read since.
fn settled(now: Instant, sent: Instant, last_read: Option<Instant>) -> bool {
  match last_read {
    Some(read) => now.duration_since(read) >= QUIET,
    None => now.duration_since(sent) >= GIVE_UP,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn waits_for_reads_to_go_quiet() {
    let sent = Instant::now();
    let read = sent + Duration::from_millis(30);
    assert!(!settled(sent + Duration::from_millis(500), sent, None));
    assert!(!settled(
      read + Duration::from_millis(100),
      sent,
      Some(read)
    ));
    assert!(settled(read + QUIET, sent, Some(read)));
    assert!(settled(sent + GIVE_UP, sent, None));
  }
}
