// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps typing and shortcuts out of Glide. A letter control pressed while
//! someone is typing types instead. Any other key pressed while a control is
//! held makes a shortcut of it: whatever glide is open is cancelled, and none
//! opens again until every control is released. The mouse control opens a
//! glide only once the pointer has travelled with it held.

use std::{
  collections::HashSet,
  sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex, MutexGuard,
  },
  time::Instant,
};

use windows::Win32::{
  Foundation::POINT,
  UI::{
    Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU, VK_RWIN},
    WindowsAndMessaging::GetCursorPos,
  },
};

use super::{key_hook, native_settings, session, spaces, InputKind};
use crate::glide::core::deliberate::{StartTravel, TypingIntent};

#[derive(Default)]
struct Typing {
  intent: TypingIntent,
  /// Letter controls that went down as typing, until they are released.
  letters: HashSet<u32>,
}

static TYPING: LazyLock<Mutex<Typing>> = LazyLock::new(Mutex::default);
static CHORDED: AtomicBool = AtomicBool::new(false);
static MOUSE_START: LazyLock<Mutex<StartTravel>> = LazyLock::new(Mutex::default);

/// Whether a keyboard hook transition types rather than drives Glide. Such a
/// key passes on untouched and is never recorded as a held control.
pub(super) fn types(key: u32, pressed: bool) -> bool {
  if !pressed {
    return typing().letters.remove(&key);
  }
  if is_typing(key) {
    return true;
  }
  let settings = native_settings::snapshot();
  if !settings.enabled || crate::shortcuts::is_capturing() {
    return false;
  }
  let control = [
    settings.mouse_modifier,
    settings.monitors_modifier,
    settings.spaces_modifier,
    settings.thirds_modifier,
  ]
  .into_iter()
  .find(|control| control.matches(key));
  let letter = match control {
    // Modifiers neither type nor count as typing; a repeat of a held control
    // keeps the role its first press took.
    Some(control) if control.is_keyboard_modifier() || native_settings::is_down(control) => {
      return false
    }
    Some(_) => true,
    None if is_modifier_key(key) || key == u32::from(VK_ESCAPE.0) => return false,
    None => false,
  };
  let pointer = cursor().unwrap_or_default();
  let now = Instant::now();
  let mut typing = typing();
  if letter {
    // A glide under way owns its controls, the thirds control among them.
    let gliding = session::active_input().is_some() || spaces::active_input().is_some();
    if gliding || (!shortcut_held() && typing.intent.claims_control(now, pointer)) {
      return false;
    }
    typing.letters.insert(key);
  }
  typing.intent.note_key(now, pointer);
  drop(typing);
  if any_control_down() {
    CHORDED.store(true, Ordering::Release);
    key_hook::post_shortcut();
  }
  letter
}

/// Whether raw input should leave this key alone: the keyboard hook has
/// already passed it on as typing.
pub(super) fn is_typing(key: u32) -> bool {
  typing().letters.contains(&key)
}

pub(super) fn note_click() {
  if let Some(pointer) = cursor() {
    typing().intent.note_click(pointer);
  }
}

/// Cancels the glide a shortcut's held control opened. Runs on the input
/// window's thread, after the keyboard hook has returned.
pub(super) fn cancel_for_shortcut() {
  if session::active_input().is_some() || spaces::active_input().is_some() {
    super::finish_current_session(true);
  }
}

/// Whether a shortcut made with a held control still keeps glides closed.
pub(super) fn blocks_sessions() -> bool {
  CHORDED.load(Ordering::Acquire)
}

/// Releases what a held control kept: the shortcut latch once every control
/// is up, and the mouse glide's origin once the mouse control is.
pub(super) fn tick() {
  if CHORDED.load(Ordering::Acquire) && !any_control_down() {
    CHORDED.store(false, Ordering::Release);
  }
  if !native_settings::is_down(native_settings::snapshot().mouse_modifier) {
    mouse_start().reset();
  }
}

/// Opens the mouse control's glide once the pointer has travelled far enough
/// with the control held, anchored where it set off.
pub(super) fn begin_mouse() -> bool {
  let Some(pointer) = cursor() else {
    return false;
  };
  let reached = mouse_start().reached(pointer);
  let Some((x, y)) = reached else {
    return false;
  };
  let anchor = POINT {
    x: x as i32,
    y: y as i32,
  };
  let began = super::begin_session_at(InputKind::Mouse, anchor);
  let mut start = mouse_start();
  if began {
    start.reset();
  } else {
    start.restart(pointer);
  }
  began
}

/// The controls that open a glide while held. Thirds only refines one.
fn any_control_down() -> bool {
  let settings = native_settings::snapshot();
  [
    settings.mouse_modifier,
    settings.monitors_modifier,
    settings.spaces_modifier,
  ]
  .into_iter()
  .any(native_settings::is_down)
}

/// Control, Alt and Windows turn a letter into a shortcut, AltGr included.
/// Shift does not: it only capitalises.
fn shortcut_held() -> bool {
  [VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
    .into_iter()
    .any(|key| unsafe { GetAsyncKeyState(i32::from(key.0)) } < 0)
}

fn is_modifier_key(key: u32) -> bool {
  matches!(key, 0x10..=0x12 | 0x14 | 0x5b | 0x5c | 0xa0..=0xa5)
}

fn cursor() -> Option<(f64, f64)> {
  let mut point = POINT::default();
  unsafe { GetCursorPos(&mut point) }
    .ok()
    .map(|()| (f64::from(point.x), f64::from(point.y)))
}

fn typing() -> MutexGuard<'static, Typing> {
  TYPING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn mouse_start() -> MutexGuard<'static, StartTravel> {
  MOUSE_START
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}
