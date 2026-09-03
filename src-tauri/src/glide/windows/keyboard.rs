// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::{
  Input::{KeyboardAndMouse::VK_ESCAPE, RAWKEYBOARD},
  WindowsAndMessaging::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP},
};

use super::{native_settings, session, InputKind, APP};

pub(super) fn handle(keyboard: RAWKEYBOARD) {
  let key = u32::from(keyboard.VKey);
  let pressed = matches!(keyboard.Message, WM_KEYDOWN | WM_SYSKEYDOWN);
  let released = matches!(keyboard.Message, WM_KEYUP | WM_SYSKEYUP);
  handle_transition(key, pressed, released);
}

pub(super) fn handle_transition(key: u32, pressed: bool, released: bool) {
  let settings = native_settings::snapshot();
  let mouse_control = native_settings::matches(settings.mouse_modifier, key);
  let configured_key = native_settings::matches(settings.mouse_modifier, key)
    || native_settings::matches(settings.thirds_modifier, key);
  if key == u32::from(VK_ESCAPE.0) && pressed && !configured_key {
    super::finish_current_session(true);
    return;
  }
  if pressed
    && mouse_control
    && !settings.mouse_modifier.is_keyboard_modifier()
    && session::active_input().is_none()
  {
    let _ = super::begin_session(InputKind::Mouse);
  }
  if session::active_input() == Some(InputKind::Mouse) && released && mouse_control {
    super::finish_current_session(false);
    return;
  }
  if session::active_input().is_some() && native_settings::matches(settings.thirds_modifier, key) {
    if let Some(app) = APP.get() {
      session::set_thirds(app, native_settings::is_down(settings.thirds_modifier));
    }
  }
}
