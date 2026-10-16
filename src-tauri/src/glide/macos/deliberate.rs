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
    LazyLock, Mutex,
  },
  time::Instant,
};

use core_graphics::{
  event::{CGEvent, CGEventFlags, CGEventType, EventField, KeyCode},
  geometry::CGPoint,
};
use tauri::AppHandle;

use super::{
  native_settings,
  session::{self, InputKind, SharedState},
  spaces,
};
use crate::glide::core::deliberate::{StartTravel, TypingIntent};

#[derive(Default)]
struct Typing {
  intent: TypingIntent,
  /// Letter controls that went down as typing, until they are released.
  letters: HashSet<i64>,
}

static TYPING: LazyLock<Mutex<Typing>> = LazyLock::new(Mutex::default);
static CHORDED: AtomicBool = AtomicBool::new(false);
static MOUSE_START: LazyLock<Mutex<StartTravel>> = LazyLock::new(Mutex::default);

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
    CGEventType::KeyUp => typing().letters.remove(&key_code(event)),
    _ => {
      let point = event.location();
      typing().intent.note_click((point.x, point.y));
      false
    }
  }
}

fn key_down(app: &AppHandle, state: &SharedState, event: &CGEvent) -> bool {
  let code = key_code(event);
  if event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0 {
    return typing().letters.contains(&code);
  }
  let settings = native_settings::snapshot();
  if !settings.enabled || crate::shortcuts::is_capturing() {
    return false;
  }
  let letter = [
    settings.mouse_modifier,
    settings.monitors_modifier,
    settings.spaces_modifier,
    settings.thirds_modifier,
  ]
  .iter()
  .any(|control| control.matches_key(code));
  if !letter && code == i64::from(KeyCode::ESCAPE) {
    return false;
  }
  let point = event.location();
  let now = Instant::now();
  let mut typing = typing();
  if letter {
    // A glide under way owns its controls, the thirds control among them.
    let gliding = session::is_active(state) || spaces::is_active();
    if gliding || (!shortcut_held(event) && typing.intent.claims_control(now, (point.x, point.y))) {
      return false;
    }
    typing.letters.insert(code);
  }
  typing.intent.note_key(now, (point.x, point.y));
  drop(typing);
  interrupt(app, state);
  letter
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

fn interrupt(app: &AppHandle, state: &SharedState) {
  if !any_control_down() {
    return;
  }
  CHORDED.store(true, Ordering::Release);
  if session::is_active(state) {
    session::end_session(app, state, true);
  }
  if spaces::is_active() {
    spaces::cancel(app);
  }
}

/// Whether a shortcut made with a held control still keeps glides closed.
pub(super) fn blocks_sessions() -> bool {
  if !CHORDED.load(Ordering::Acquire) {
    return false;
  }
  if any_control_down() {
    return true;
  }
  CHORDED.store(false, Ordering::Release);
  false
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
pub(super) fn begin_mouse(app: &AppHandle, state: &SharedState, pointer: CGPoint) -> bool {
  let reached = mouse_start().reached((pointer.x, pointer.y));
  let Some((x, y)) = reached else {
    return false;
  };
  let began = session::begin_if_titlebar(app, state, InputKind::Mouse, CGPoint::new(x, y));
  let mut start = mouse_start();
  if began {
    start.reset();
  } else {
    start.restart((pointer.x, pointer.y));
  }
  began
}

pub(super) fn reset_mouse_start() {
  mouse_start().reset();
}

fn key_code(event: &CGEvent) -> i64 {
  event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
}

fn typing() -> std::sync::MutexGuard<'static, Typing> {
  TYPING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn mouse_start() -> std::sync::MutexGuard<'static, StartTravel> {
  MOUSE_START
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}
