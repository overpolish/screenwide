// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{lifecycle, transport, Input, CLOSING, EVENT_TAG, SESSION};
use crate::glide::platform::{
  native_settings,
  session::{self, SharedState},
};
use core_graphics::event::{CGEvent, CGEventType, CallbackResult, EventField};
use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLED_UNTIL_RELEASE: AtomicBool = AtomicBool::new(false);
use tauri::AppHandle;

pub(super) fn cancelled_until_release() -> bool {
  CANCELLED_UNTIL_RELEASE.load(Ordering::Acquire)
}

pub(super) fn clear_cancelled_if_released() {
  if CANCELLED_UNTIL_RELEASE.load(Ordering::Acquire)
    && !native_settings::is_down(native_settings::snapshot().spaces_modifier)
  {
    CANCELLED_UNTIL_RELEASE.store(false, Ordering::Release);
  }
}

pub(in crate::glide::platform) fn handle_event(
  app: &AppHandle,
  normal: &SharedState,
  kind: CGEventType,
  event: &CGEvent,
) -> Option<CallbackResult> {
  if event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA) == EVENT_TAG {
    return Some(CallbackResult::Keep);
  }
  if CANCELLED_UNTIL_RELEASE.load(Ordering::Acquire) {
    if !native_settings::is_down(native_settings::snapshot().spaces_modifier) {
      CANCELLED_UNTIL_RELEASE.store(false, Ordering::Release);
    } else if !matches!(
      kind,
      CGEventType::KeyDown | CGEventType::KeyUp | CGEventType::FlagsChanged
    ) {
      return Some(CallbackResult::Drop);
    }
  }
  if CLOSING.load(Ordering::Acquire) {
    return Some(
      if matches!(
        kind,
        CGEventType::KeyDown | CGEventType::KeyUp | CGEventType::FlagsChanged
      ) || (matches!(kind, CGEventType::MouseMoved)
        && !crate::glide::platform::cursor::is_cursor_pinned())
      {
        CallbackResult::Keep
      } else {
        CallbackResult::Drop
      },
    );
  }
  let active = SESSION.lock().ok().and_then(|state| {
    state
      .as_ref()
      .map(|session| (session.input, session.moving, session.armed))
  });
  let settings = native_settings::snapshot();
  let spaces_down = native_settings::is_down(settings.spaces_modifier);
  if let Some((input, moving, armed)) = active {
    if matches!(kind, CGEventType::KeyDown)
      && event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) == 53
    {
      if input == Input::Trackpad {
        session::set_momentum_suppression(normal, true);
      }
      CANCELLED_UNTIL_RELEASE.store(true, Ordering::Release);
      lifecycle::request_end(app, true);
      return Some(CallbackResult::Drop);
    }
    if !spaces_down || !settings.enabled {
      if input == Input::Trackpad {
        session::set_momentum_suppression(normal, true);
      }
      lifecycle::request_end(app, !settings.enabled);
      return Some(
        if matches!(kind, CGEventType::FlagsChanged | CGEventType::KeyUp) {
          CallbackResult::Keep
        } else {
          CallbackResult::Drop
        },
      );
    }
    if matches!(kind, CGEventType::ScrollWheel) {
      if event.get_integer_value_field(123) != 0 {
        return Some(CallbackResult::Drop);
      }
      if input != Input::Mouse {
        let phase = event.get_integer_value_field(99);
        let continuous = event.get_integer_value_field(88);
        let (delta_x, delta_y) = if continuous == 0 {
          (
            event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2),
            event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1),
          )
        } else {
          (
            event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2),
            event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1),
          )
        };
        if armed
          && !crate::glide::core::armed::can_claim_scroll(
            true,
            phase,
            event.get_integer_value_field(123),
            delta_x,
            delta_y,
          )
        {
          return Some(CallbackResult::Drop);
        }
        let effective_input = if input == Input::Wheel {
          let claimed_input = if event.get_integer_value_field(88) == 0 {
            Input::Wheel
          } else {
            Input::Trackpad
          };
          if lifecycle::claim_armed(claimed_input) {
            claimed_input
          } else {
            input
          }
        } else {
          input
        };
        sample_scroll(app, event, effective_input);
        let phase = event.get_integer_value_field(99);
        if effective_input == Input::Trackpad && phase & (4 | 8) != 0 {
          session::set_momentum_suppression(normal, true);
          lifecycle::request_end(app, phase & 8 != 0);
        }
      }
      return Some(CallbackResult::Drop);
    }
    if matches!(
      kind,
      CGEventType::MouseMoved | CGEventType::OtherMouseDragged | CGEventType::LeftMouseDragged
    ) {
      let mouse_dx = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X);
      let mouse_dy = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y);
      if input == Input::Wheel
        && (mouse_dx != 0 || mouse_dy != 0)
        && !crate::glide::platform::multitouch::pointer_episode_active()
        && lifecycle::promote_armed_to_mouse()
      {
        transport::update(app, mouse_dx as f64, mouse_dy as f64);
        return Some(CallbackResult::Drop);
      }
      if input == Input::Mouse {
        transport::update(
          app,
          event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64,
          event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64,
        );
      }
      return Some(CallbackResult::Drop);
    }
    if matches!(kind, CGEventType::LeftMouseDown | CGEventType::LeftMouseUp) {
      if !moving {
        CANCELLED_UNTIL_RELEASE.store(true, Ordering::Release);
        lifecycle::request_end(app, true);
      }
      return Some(CallbackResult::Drop);
    }
    return Some(CallbackResult::Keep);
  }
  if CANCELLED_UNTIL_RELEASE.load(Ordering::Acquire)
    || !spaces_down
    || !settings.enabled
    || session::is_active(normal)
    || crate::shortcuts::is_capturing()
    || crate::capture_overlays::blocks_glide(app)
  {
    return None;
  }
  let input = match kind {
    CGEventType::ScrollWheel => {
      if event.get_integer_value_field(123) != 0 {
        return Some(CallbackResult::Drop);
      }
      if event.get_integer_value_field(99) & (4 | 8) != 0 {
        return Some(CallbackResult::Drop);
      }
      if event.get_integer_value_field(88) == 0 {
        Input::Wheel
      } else {
        Input::Trackpad
      }
    }
    CGEventType::MouseMoved | CGEventType::OtherMouseDragged => Input::Mouse,
    _ => return None,
  };
  if !lifecycle::begin(app, input, event.location()) {
    return Some(CallbackResult::Keep);
  }
  if input == Input::Mouse {
    transport::update(
      app,
      event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64,
      event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64,
    );
  } else {
    sample_scroll(app, event, input);
  }
  Some(CallbackResult::Drop)
}

fn sample_scroll(app: &AppHandle, event: &CGEvent, input: Input) {
  if input == Input::Wheel {
    let horizontal =
      event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2) as f64;
    let vertical =
      event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1) as f64;
    // Wheel up means previous; sideways wheels use the same logical order.
    let steps = if horizontal != 0.0 {
      -horizontal
    } else {
      -vertical
    };
    transport::update(app, steps * 36.0, 0.0);
  } else {
    transport::update(
      app,
      event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2) as f64,
      event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1) as f64,
    );
  }
}
