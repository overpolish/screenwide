// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps typing and shortcuts out of Glide, on every platform. A letter
//! control types instead when someone is typing, or when Glide could not hold
//! it back from the application: a key the application has received cannot
//! also drive a glide. Any other key pressed while a control is held makes a
//! shortcut of it: whatever glide is open is cancelled, and none opens again
//! until every control is released.
//! The mouse control opens a glide only once the pointer has travelled with
//! it held. A platform reads its own keys and pointer and asks here.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::Instant;

use super::deliberate::{StartTravel, TypingIntent};

/// What a key is to Glide, as the platform reads it against the controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyRole {
  /// A Glide control that types as well: a letter or a symbol key.
  Letter,
  /// Any other key that types or starts a shortcut.
  Other,
  /// A modifier, Escape, or a held control repeating: neither typing nor a
  /// shortcut.
  Ignored,
}

/// What a key going down came to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct KeyDown {
  /// The key types: it passes on untouched and never counts as a held
  /// control.
  pub types: bool,
  /// The key made a shortcut of a held control: the platform cancels
  /// whatever glide is open.
  pub shortcut: bool,
}

#[derive(Default)]
struct Typing {
  intent: TypingIntent,
  /// Letter controls that went down as typing, until they are released.
  letters: HashSet<u32>,
}

static TYPING: LazyLock<Mutex<Typing>> = LazyLock::new(Mutex::default);
static CHORDED: AtomicBool = AtomicBool::new(false);
static MOUSE_START: LazyLock<Mutex<StartTravel>> = LazyLock::new(Mutex::default);

fn typing() -> MutexGuard<'static, Typing> {
  TYPING.lock().unwrap_or_else(|error| error.into_inner())
}

fn mouse_start() -> MutexGuard<'static, StartTravel> {
  MOUSE_START
    .lock()
    .unwrap_or_else(|error| error.into_inner())
}

/// A key went down. `repeat` when the platform knows it for an auto-repeat;
/// `shortcut_held` when Command, Ctrl, Alt or the Windows key is down;
/// `gliding` when a glide is under way; `controls_down` when any control that
/// opens a glide is held; `reservable` when the platform would hold this key
/// back from the application if Glide took it; `pointer` where the pointer
/// is.
#[allow(clippy::too_many_arguments)]
pub(crate) fn key_down(
  key: u32,
  repeat: bool,
  role: KeyRole,
  shortcut_held: bool,
  gliding: bool,
  controls_down: bool,
  reservable: bool,
  pointer: (f64, f64),
) -> KeyDown {
  let mut typing = typing();
  // A key already typing keeps typing until it comes up, repeats included.
  if repeat || typing.letters.contains(&key) {
    return KeyDown {
      types: typing.letters.contains(&key),
      shortcut: false,
    };
  }
  if role == KeyRole::Ignored {
    return KeyDown::default();
  }
  let now = Instant::now();
  let letter = role == KeyRole::Letter;
  if letter {
    // A glide under way owns its controls, the thirds control among them.
    if gliding || (!shortcut_held && reservable && typing.intent.claims_control(now, pointer)) {
      return KeyDown::default();
    }
    typing.letters.insert(key);
  }
  typing.intent.note_key(now, pointer);
  drop(typing);
  let shortcut = controls_down;
  if shortcut {
    CHORDED.store(true, Ordering::Release);
  }
  KeyDown {
    types: letter,
    shortcut,
  }
}

/// A key came up. Answers whether it was typing.
pub(crate) fn key_up(key: u32) -> bool {
  typing().letters.remove(&key)
}

/// Whether `key` is down as typing, which the platform's other input paths
/// leave alone.
pub(crate) fn is_typing(key: u32) -> bool {
  typing().letters.contains(&key)
}

pub(crate) fn note_click(pointer: (f64, f64)) {
  typing().intent.note_click(pointer);
}

/// Whether a shortcut made with a held control still keeps glides closed.
/// The latch lets go once every control is up.
pub(crate) fn blocks_sessions(controls_down: bool) -> bool {
  if !CHORDED.load(Ordering::Acquire) {
    return false;
  }
  if controls_down {
    return true;
  }
  CHORDED.store(false, Ordering::Release);
  false
}

/// Forgets where the mouse control's glide set off, once that control is up.
pub(crate) fn reset_mouse_start() {
  mouse_start().reset();
}

/// Where the mouse control's glide opens: once the pointer has travelled far
/// enough with the control held, the point it set off from. `begin` opens
/// the glide there and answers whether it did; a glide that did not open
/// starts its travel again from `pointer`.
pub(crate) fn begin_mouse(pointer: (f64, f64), begin: impl FnOnce((f64, f64)) -> bool) -> bool {
  let reached = mouse_start().reached(pointer);
  let Some(anchor) = reached else {
    return false;
  };
  let began = begin(anchor);
  let mut start = mouse_start();
  if began {
    start.reset();
  } else {
    start.restart(pointer);
  }
  began
}
