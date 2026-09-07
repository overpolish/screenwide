// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;

use super::{native_settings, session, InputKind, APP};

pub(super) fn handle_transition(key: u32, pressed: bool, released: bool) {
  let settings = native_settings::snapshot();
  let mouse_control = native_settings::matches(settings.mouse_modifier, key);
  let monitor_control = native_settings::matches(settings.monitors_modifier, key);
  let configured_key = native_settings::matches(settings.mouse_modifier, key)
    || monitor_control
    || native_settings::matches(settings.spaces_modifier, key)
    || native_settings::matches(settings.thirds_modifier, key);
  if key == u32::from(VK_ESCAPE.0)
    && pressed
    && !configured_key
    && (session::active_input().is_some() || super::spaces::active_input().is_some())
  {
    super::navigation::cancel_until_release();
    super::finish_current_session(true);
    return;
  }
  if pressed
    && mouse_control
    && !settings.mouse_modifier.is_keyboard_modifier()
    && native_settings::is_down(settings.mouse_modifier)
    && session::active_input().is_none()
  {
    let _ = super::begin_session(InputKind::Mouse);
  }
  if released && monitor_control {
    session::clear_monitor_commit();
  }
  if released && native_settings::matches(settings.spaces_modifier, key) {
    super::spaces::clear_committed();
  }
  if session::monitor_mode()
    && released
    && monitor_control
    && !native_settings::is_down(settings.monitors_modifier)
  {
    super::finish_current_session(false);
    return;
  }
  if session::active_input() == Some(InputKind::Mouse)
    && released
    && mouse_control
    && !session::monitor_mode()
    && !native_settings::is_down(settings.mouse_modifier)
  {
    super::finish_current_session(false);
    return;
  }
  if session::active_input().is_some() && native_settings::matches(settings.thirds_modifier, key) {
    if let Some(app) = APP.get() {
      session::set_thirds(app, native_settings::is_down(settings.thirds_modifier));
    }
  }
}
