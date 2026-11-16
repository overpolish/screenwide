// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text put where the caret is, the way a person would: on the clipboard,
//! then the paste keys. Typing it key by key is slow and garbles characters
//! in some Windows apps; pasting is what every app takes.
//!
//! It goes to the app that was in front when the text was asked for
//! ([`Target`]). A press on Screenwide's own windows meanwhile, such as the
//! island's Done, brings Screenwide in front, so that app is brought back
//! first; one the person switched to themselves keeps the paste.
//!
//! The text stays on the clipboard afterwards, as text copied does, and what
//! the clipboard held before is not put back. Whether a paste landed cannot
//! be told for certain: an app reads the clipboard when the paste keys are
//! pressed whether or not anything there takes text (Safari's page, Finder's
//! files). Left on the clipboard, text that went nowhere is never lost, and
//! the focused element says when that is likely ([`Pasted::OnClipboard`]).

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

/// Where the text went. Either way it is on the clipboard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pasted {
  /// The paste keys went to something that takes text, or to an app that
  /// does not say what has the keyboard.
  Typed,
  /// Nothing that takes text had the keyboard, or the paste keys could not
  /// be pressed: the text waits on the clipboard for the person to paste.
  OnClipboard,
}

/// Why the text was not pasted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
  /// A password field has the keyboard, and nothing typed for it may pass
  /// through the clipboard. The clipboard is left alone.
  SecureField,
  /// The app in front runs as administrator, and Windows lets nothing less
  /// privileged send it keys. The text is on the clipboard.
  Elevated,
  /// The text could not be put on the clipboard; the words are English, for
  /// the log.
  Failed(String),
}

/// Runs work on the app's main thread, which macOS's clipboard needs.
pub type OnMain<'a> = &'a dyn Fn(Box<dyn FnOnce() + Send>);

/// How long a target brought back is given to come in front.
const COMING_FORWARD: Duration = Duration::from_secs(1);
/// How often it is looked for in front meanwhile.
const POLL: Duration = Duration::from_millis(20);

/// The app in front, to paste into later.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Target(platform::Target);

impl Target {
  /// The app in front now, unless it is Screenwide.
  pub fn current() -> Option<Self> {
    platform::Target::current().map(Self)
  }
}

/// Puts `text` on the clipboard and pastes it into the app in front,
/// bringing `target` back first when Screenwide has come in front meanwhile.
/// Must not run on the main thread, as it waits for the target; work that
/// needs that thread is handed to `on_main`.
pub fn paste(text: &str, target: Option<Target>, on_main: OnMain<'_>) -> Result<Pasted, Refusal> {
  if let Some(Target(target)) = target {
    if platform::bring_back(target, on_main) {
      let deadline = Instant::now() + COMING_FORWARD;
      while !platform::is_in_front(target, on_main) && Instant::now() < deadline {
        std::thread::sleep(POLL);
      }
    }
  }
  platform::paste(text, on_main)
}

/// What has the keyboard in the app in front, as its accessibility says.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
  /// A field, a document or anything else with text to type into.
  TakesText,
  /// Something with no text to type into: a page, a list of files.
  Elsewhere,
  /// The app says nothing, as many do until asked to.
  Unknown,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl Focus {
  /// Where text pasted with this focus went.
  const fn pasted(self) -> Pasted {
    match self {
      Self::Elsewhere => Pasted::OnClipboard,
      Self::TakesText | Self::Unknown => Pasted::Typed,
    }
  }
}
