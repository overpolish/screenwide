// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows' half of keeping typing and shortcuts out of Glide: virtual-key
//! codes and the cursor read for the shared guard in
//! `glide::core::typing_guard`.

use windows::Win32::{
  Foundation::POINT,
  UI::{
    Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU, VK_RWIN},
    WindowsAndMessaging::GetCursorPos,
  },
};

use super::{key_hook, native_settings, session, spaces, InputKind};
use crate::glide::core::typing_guard::{self as guard, KeyRole};

/// Whether a keyboard hook transition types rather than drives Glide. Such a
/// key passes on untouched and is never recorded as a held control.
pub(super) fn types(key: u32, pressed: bool) -> bool {
  if !pressed {
    return guard::key_up(key);
  }
  if guard::is_typing(key) {
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
  let role = match control {
    // Modifiers neither type nor count as typing; a repeat of a held control
    // keeps the role its first press took.
    Some(control) if control.is_keyboard_modifier() || native_settings::is_down(control) => {
      KeyRole::Ignored
    }
    Some(_) => KeyRole::Letter,
    None if is_modifier_key(key) || key == u32::from(VK_ESCAPE.0) => KeyRole::Ignored,
    None => KeyRole::Other,
  };
  let gliding = session::active_input().is_some() || spaces::active_input().is_some();
  let outcome = guard::key_down(
    key,
    false,
    role,
    shortcut_held(),
    gliding,
    any_control_down(),
    key_hook::suppresses(key),
    cursor().unwrap_or_default(),
  );
  if outcome.shortcut {
    key_hook::post_shortcut();
  }
  outcome.types
}

/// Whether raw input should leave this key alone: the keyboard hook has
/// already passed it on as typing.
pub(super) fn is_typing(key: u32) -> bool {
  guard::is_typing(key)
}

pub(super) fn note_click() {
  if let Some(pointer) = cursor() {
    guard::note_click(pointer);
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
  guard::blocks_sessions(any_control_down())
}

/// Releases what a held control kept: the shortcut latch once every control
/// is up, and the mouse glide's origin once the mouse control is.
pub(super) fn tick() {
  guard::blocks_sessions(any_control_down());
  if !native_settings::is_down(native_settings::snapshot().mouse_modifier) {
    guard::reset_mouse_start();
  }
}

/// Opens the mouse control's glide once the pointer has travelled far enough
/// with the control held, anchored where it set off.
pub(super) fn begin_mouse() -> bool {
  let Some(pointer) = cursor() else {
    return false;
  };
  guard::begin_mouse(pointer, |(x, y)| {
    super::begin_session_at(
      InputKind::Mouse,
      POINT {
        x: x as i32,
        y: y as i32,
      },
    )
  })
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
