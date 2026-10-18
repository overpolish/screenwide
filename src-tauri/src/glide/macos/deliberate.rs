// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS's half of keeping typing and shortcuts out of Glide: the event tap's
//! key codes, flags and locations read for the shared guard in
//! `glide::core::typing_guard`.

use core_graphics::{
  event::{CGEvent, CGEventFlags, CGEventType, EventField, KeyCode},
  geometry::CGPoint,
};
use tauri::AppHandle;

use super::{
  native_settings,
  own_window::any_titlebar,
  session::{self, InputKind, SharedState},
  spaces,
};
use crate::glide::core::typing_guard::{self as guard, KeyRole};

/// Whether this event types rather than drives Glide. Such an event passes on
/// untouched, unseen by every other handler, so it never counts as a held
/// control either.
pub(super) fn types(
  app: &AppHandle,
  state: &SharedState,
  event_type: CGEventType,
  event: &CGEvent,
) -> bool {
  let tracked = matches!(
    event_type,
    CGEventType::LeftMouseDown
      | CGEventType::OtherMouseDown
      | CGEventType::KeyDown
      | CGEventType::KeyUp
  );
  // Spaces posts its own tagged keystrokes to switch desktops.
  if !tracked || spaces::is_synthetic(event) {
    return false;
  }
  match event_type {
    CGEventType::KeyDown => key_down(app, state, event),
    CGEventType::KeyUp => guard::key_up(key_code(event)),
    _ => {
      let point = event.location();
      guard::note_click((point.x, point.y));
      false
    }
  }
}

fn key_down(app: &AppHandle, state: &SharedState, event: &CGEvent) -> bool {
  let code = key_code(event);
  let repeat = event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;
  let settings = native_settings::snapshot();
  if !repeat && (!settings.enabled || crate::shortcuts::is_capturing()) {
    return false;
  }
  let letter = [
    settings.mouse_modifier,
    settings.monitors_modifier,
    settings.spaces_modifier,
    settings.thirds_modifier,
  ]
  .iter()
  .any(|control| control.matches_key(i64::from(code)));
  let role = if letter {
    KeyRole::Letter
  } else if i64::from(code) == i64::from(KeyCode::ESCAPE) {
    KeyRole::Ignored
  } else {
    KeyRole::Other
  };
  let point = event.location();
  // The event tap holds a letter control back only over a title bar; a glide
  // under way is the guard's to decide.
  let reservable = letter && !repeat && any_titlebar(app, point);
  let outcome = guard::key_down(
    code,
    repeat,
    role,
    shortcut_held(event),
    session::is_active(state) || spaces::is_active(),
    any_control_down(),
    reservable,
    (point.x, point.y),
  );
  if outcome.shortcut {
    if session::is_active(state) {
      session::end_session(app, true);
    }
    if spaces::is_active() {
      spaces::cancel(app);
    }
  }
  outcome.types
}

/// Command, Control and Option turn a letter into a shortcut. Shift does not:
/// it only capitalises.
fn shortcut_held(event: &CGEvent) -> bool {
  event.get_flags().intersects(
    CGEventFlags::CGEventFlagCommand
      | CGEventFlags::CGEventFlagControl
      | CGEventFlags::CGEventFlagAlternate,
  )
}

/// Whether a shortcut made with a held control still keeps glides closed.
pub(super) fn blocks_sessions() -> bool {
  guard::blocks_sessions(any_control_down())
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

/// Opens the mouse control's glide once the pointer has travelled far enough
/// with the control held, anchored where it set off.
pub(super) fn begin_mouse(app: &AppHandle, pointer: CGPoint) -> bool {
  guard::begin_mouse((pointer.x, pointer.y), |(x, y)| {
    session::begin_if_titlebar(app, InputKind::Mouse, CGPoint::new(x, y))
  })
}

pub(super) fn reset_mouse_start() {
  guard::reset_mouse_start();
}

/// The key code, which fits in 16 bits on macOS.
fn key_code(event: &CGEvent) -> u32 {
  u32::try_from(event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)).unwrap_or(0)
}
