// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The process's one low-level keyboard hook on Windows. Every feature that
//! needs the keyboard outside Screenwide's own windows (the system's
//! screenshot keys, Glide, the Ruler and Text Recognition overlays, the keys
//! a recording shows, a cursor scrub's Escape) registers a handler here
//! instead of installing a hook of its own. Windows calls the newest hook
//! first, so with a hook each, which feature won a key shared by two of them
//! came down to the order they happened to start in; here it is the fixed
//! order of [`Priority`].
//!
//! Nothing in the process registers raw keyboard input either: while it did,
//! Windows called none of the process's low-level keyboard hooks for keys
//! typed into one of its own windows.

// The routing rules are tested on every platform; only Windows uses them.
#[cfg(any(target_os = "windows", test))]
mod router;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::{observe, send_mask_key, set_taking_paused, take, Registration};

/// Which handler a key is offered to first. A key goes to the first handler
/// that takes it; one that passes leaves it to the next and, after the last,
/// to the system.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Priority {
  /// An open Ruler or Text Recognition overlay, whose keys belong to it while
  /// it is up.
  Overlay,
  /// The system's screenshot keys, taken for Screenwide's tools.
  SystemKeys,
  /// Glide's controls, and Escape while a glide is under way.
  Glide,
}

/// The modifiers held as a key arrives, followed from the hook's own key
/// stream rather than read back from the system.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
  pub shift: bool,
  pub control: bool,
  pub alt: bool,
  pub win: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
  Press,
  /// The key held down, auto-repeating.
  Repeat,
  Release,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
  /// The virtual key.
  pub key: u32,
  pub phase: Phase,
  /// Held before this event; a modifier's own press is not in it yet.
  pub modifiers: Modifiers,
  /// The key's timestamp, in the system's milliseconds.
  pub time: u32,
  pub extended: bool,
  /// Sent by software. Remote-desktop and streaming hosts (Parsec,
  /// Sunshine, Remote Desktop) deliver the user's own typing this way, so it
  /// is routed like any other key.
  pub injected: bool,
  /// Whether a handler may take this key. False while a shortcut field in
  /// Settings is recording, which needs every key to reach it.
  pub may_take: bool,
  /// Whether this handler took the key's press, so the repeat or release is
  /// its own. A key's repeats and release go only to the handler that took
  /// it, and are taken whatever it answers.
  pub owned: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
  Pass,
  Take,
}
