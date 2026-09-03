// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use core_graphics::event::{CGEvent, CGEventType, CallbackResult, EventField, KeyCode};
use tauri::AppHandle;

use super::{
  native_settings,
  own_window::any_titlebar,
  session::{
    active_input, end_session, is_active, set_detector_thirds, set_suppression, InputKind,
    SharedState,
  },
};

pub(super) fn handle_key(
  app: &AppHandle,
  state: &SharedState,
  event_type: CGEventType,
  event: &CGEvent,
) -> CallbackResult {
  if crate::shortcuts::is_capturing() {
    return CallbackResult::Keep;
  }
  let settings = native_settings::snapshot();
  let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
  let mouse_key = settings.mouse_modifier.matches_key(code);
  let thirds_key = settings.thirds_modifier.matches_key(code);
  let active = active_input(state);
  let mouse_reserved =
    mouse_key && (active == Some(InputKind::Mouse) || any_titlebar(app, event.location()));
  let thirds_reserved = thirds_key && active.is_some();
  if matches!(event_type, CGEventType::KeyDown)
    && code == i64::from(KeyCode::ESCAPE)
    && !mouse_key
    && !thirds_key
  {
    if let Some(input) = active_input(state) {
      end_session(app, state, true);
      set_suppression(state, input, true);
      return CallbackResult::Drop;
    }
  }
  if mouse_key && matches!(event_type, CGEventType::KeyUp) {
    if active_input(state) == Some(InputKind::Mouse) {
      end_session(app, state, false);
    }
    set_suppression(state, InputKind::Mouse, false);
  }
  if thirds_key && is_active(state) {
    set_detector_thirds(
      app,
      state,
      native_settings::is_down(settings.thirds_modifier),
    );
  }
  if settings.enabled && (mouse_reserved || thirds_reserved) {
    CallbackResult::Drop
  } else {
    CallbackResult::Keep
  }
}

pub(super) fn handle_mouse_button(
  app: &AppHandle,
  state: &SharedState,
  event_type: CGEventType,
  event: &CGEvent,
) -> CallbackResult {
  if crate::shortcuts::is_capturing() {
    return CallbackResult::Keep;
  }
  let settings = native_settings::snapshot();
  let button = event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
  let mouse_control = settings.mouse_modifier.matches_mouse(button);
  let thirds_control = settings.thirds_modifier.matches_mouse(button);
  let active = active_input(state);
  let mouse_reserved =
    mouse_control && (active == Some(InputKind::Mouse) || any_titlebar(app, event.location()));
  let thirds_reserved = thirds_control && active.is_some();
  if mouse_control && matches!(event_type, CGEventType::OtherMouseUp) {
    if active == Some(InputKind::Mouse) {
      end_session(app, state, false);
    }
    set_suppression(state, InputKind::Mouse, false);
  }
  if thirds_control && active.is_some() {
    set_detector_thirds(
      app,
      state,
      native_settings::is_down(settings.thirds_modifier),
    );
  }
  if settings.enabled && (mouse_reserved || thirds_reserved) {
    CallbackResult::Drop
  } else {
    CallbackResult::Keep
  }
}

pub(super) fn handle_flags_changed(app: &AppHandle, state: &SharedState) -> CallbackResult {
  let modifier_down = native_settings::is_down(native_settings::snapshot().mouse_modifier);
  if !modifier_down {
    if active_input(state) == Some(InputKind::Mouse) {
      end_session(app, state, false);
    }
    set_suppression(state, InputKind::Mouse, false);
  }
  if is_active(state) {
    let thirds = native_settings::is_down(native_settings::snapshot().thirds_modifier);
    set_detector_thirds(app, state, thirds);
  }
  CallbackResult::Keep
}
