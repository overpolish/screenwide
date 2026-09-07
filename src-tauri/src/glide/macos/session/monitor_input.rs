// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A held control selects monitor movement before any resize gesture begins.
use super::{
  active_input, begin_if_titlebar, end_session, is_active, session_anchor,
  set_momentum_suppression, update_detector, InputKind, SharedState,
};
use crate::glide::platform::{multitouch, native_settings, own_window::any_titlebar};
use core_graphics::event::{CGEvent, CGEventType, CallbackResult, EventField};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;

static RESERVED: AtomicBool = AtomicBool::new(false);
static CANCELLED: AtomicBool = AtomicBool::new(false);

pub(in crate::glide::platform) fn handle_event(
  app: &AppHandle,
  state: &SharedState,
  kind: CGEventType,
  event: &CGEvent,
) -> Option<CallbackResult> {
  let settings = native_settings::snapshot();
  let down = native_settings::is_down(settings.monitors_modifier);
  let key = matches!(kind, CGEventType::KeyDown | CGEventType::KeyUp)
    && settings
      .monitors_modifier
      .matches_key(event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE));
  let button = matches!(
    kind,
    CGEventType::OtherMouseDown | CGEventType::OtherMouseUp
  ) && settings
    .monitors_modifier
    .matches_mouse(event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER));
  let active = state.lock().is_ok_and(|state| {
    state
      .session
      .as_ref()
      .is_some_and(|session| session.monitors.is_some())
  });
  let reserved = RESERVED.load(Ordering::Relaxed);
  let cancelled = CANCELLED.load(Ordering::Relaxed);
  if !down {
    RESERVED.store(false, Ordering::Relaxed);
    CANCELLED.store(false, Ordering::Relaxed);
    if active {
      finish(app, state, !settings.enabled);
    }
    return ((key || button) && reserved).then_some(CallbackResult::Drop);
  }
  if !active
    && (is_active(state)
      || !settings.enabled
      || crate::shortcuts::is_capturing()
      || crate::capture_overlays::blocks_glide(app))
  {
    return None;
  }
  if key || button {
    if reserved || active || any_titlebar(app, event.location()) {
      RESERVED.store(true, Ordering::Relaxed);
      return Some(CallbackResult::Drop);
    }
    return None;
  }
  if matches!(kind, CGEventType::LeftMouseUp) && super::take_mouse_up_swallow(state) {
    return Some(CallbackResult::Drop);
  }
  if !active
    && matches!(kind, CGEventType::ScrollWheel)
    && super::is_suppressing(state, InputKind::Trackpad)
  {
    if event.get_integer_value_field(99) & (4 | 8) != 0 {
      super::set_suppression(state, InputKind::Trackpad, false);
    }
    return Some(CallbackResult::Drop);
  }
  if cancelled {
    return Some(if matches!(kind, CGEventType::ScrollWheel) {
      CallbackResult::Drop
    } else {
      CallbackResult::Keep
    });
  }
  if active
    && matches!(kind, CGEventType::KeyDown)
    && event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) == 53
  {
    CANCELLED.store(true, Ordering::Relaxed);
    finish(app, state, true);
    return Some(CallbackResult::Drop);
  }
  let input = match kind {
    CGEventType::ScrollWheel => {
      if event.get_integer_value_field(123) != 0 {
        return Some(CallbackResult::Drop);
      }
      if !active && event.get_integer_value_field(99) & (4 | 8) != 0 {
        return Some(CallbackResult::Drop);
      }
      if event.get_integer_value_field(88) == 0 {
        InputKind::Wheel
      } else {
        InputKind::Trackpad
      }
    }
    CGEventType::MouseMoved | CGEventType::OtherMouseDragged => {
      // Contact drift must not turn a two-finger scroll into mouse mode.
      if multitouch::pointer_episode_active() {
        return Some(CallbackResult::Keep);
      }
      InputKind::Mouse
    }
    CGEventType::LeftMouseDown if active => {
      CANCELLED.store(true, Ordering::Relaxed);
      finish(app, state, true);
      super::set_mouse_up_swallow(state, true);
      return Some(CallbackResult::Drop);
    }
    _ => return active.then_some(CallbackResult::Keep),
  };
  if !active && !begin_if_titlebar(app, state, input, event.location()) {
    // A missing destination must never fall through into resizing.
    return Some(CallbackResult::Keep);
  }
  if active_input(state) != Some(input) {
    return Some(CallbackResult::Drop);
  }
  let (x, y) = deltas(event, input);
  if input == InputKind::Mouse {
    if let Some(anchor) = session_anchor(state) {
      event.set_location(anchor);
    }
  }
  if x != 0.0 || y != 0.0 {
    update_detector(app, state, x, y, false);
  }
  let phase = event.get_integer_value_field(99);
  if input == InputKind::Trackpad && phase & (4 | 8) != 0 {
    finish(app, state, phase & 8 != 0);
    super::set_suppression(state, InputKind::Trackpad, false);
  }
  Some(CallbackResult::Drop)
}

fn finish(app: &AppHandle, state: &SharedState, cancelled: bool) {
  if active_input(state) == Some(InputKind::Trackpad) {
    set_momentum_suppression(state, true);
    // Swallow an unfinished finger episode if the control was released first.
    super::set_suppression(state, InputKind::Trackpad, true);
  }
  end_session(app, state, cancelled);
}

fn deltas(event: &CGEvent, input: InputKind) -> (f64, f64) {
  let (x, y, scale) = match input {
    InputKind::Mouse => (
      EventField::MOUSE_EVENT_DELTA_X,
      EventField::MOUSE_EVENT_DELTA_Y,
      1.0,
    ),
    InputKind::Trackpad => (
      EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2,
      EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1,
      1.0,
    ),
    InputKind::Wheel => (
      EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2,
      EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1,
      -36.0,
    ),
  };
  (
    event.get_integer_value_field(x) as f64 * scale,
    event.get_integer_value_field(y) as f64 * scale,
  )
}
