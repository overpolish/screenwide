// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
  CallbackResult, EventField,
};
use tauri::AppHandle;

#[path = "macos/center.rs"]
mod center;
#[path = "macos/commands.rs"]
mod commands;
#[path = "macos/control.rs"]
mod control;
#[path = "macos/control_keys.rs"]
mod control_keys;
#[path = "macos/cursor.rs"]
mod cursor;
#[path = "macos/key.rs"]
mod key;
#[path = "macos/multitouch.rs"]
mod multitouch;
#[path = "macos/native_settings.rs"]
mod native_settings;
#[path = "macos/own_window.rs"]
mod own_window;
#[path = "macos/session.rs"]
mod session;
#[path = "macos/titlebar.rs"]
mod titlebar;
#[path = "macos/tween.rs"]
mod tween;

use center::center_window_at;
pub(super) use commands::haptic;
use own_window::any_titlebar;
use session::{
  accumulate_pointer_travel, active_input, begin_if_titlebar, end_session, is_active,
  is_suppressing, is_suppressing_momentum, session_anchor, set_detector_thirds,
  set_momentum_suppression, set_mouse_up_swallow, set_suppression, settle_detector,
  take_mouse_up_swallow, update_detector, InputKind, SharedState,
};

const POLL_INTERVAL: Duration = Duration::from_millis(16);
const POINTER_DISMISS_DISTANCE: f64 = 15.0;
const SCROLL_PHASE_FIELD: u32 = 99;
const SCROLL_MOMENTUM_PHASE_FIELD: u32 = 123;
const SCROLL_IS_CONTINUOUS_FIELD: u32 = 88;
// The phase field carries CGScrollPhase values, not the NSEventPhase ones a
// gesture recogniser would see: Began=1, Changed=2, Ended=4, Cancelled=8.
const SCROLL_PHASE_ENDED: i64 = 4;
const SCROLL_PHASE_CANCELLED: i64 = 8;
/// The click state a double click carries, as the second press of the pair.
const DOUBLE_CLICK_STATE: i64 = 2;

/// Hands the saved settings to the tap's snapshot. The tap itself keeps running
/// whatever they say: a CGEventTap cannot be cheaply stopped and started again,
/// and one that passes every event through is behaviour-identical to none.
pub(super) fn apply_settings(settings: &crate::glide::settings::GlideSettings) {
  native_settings::apply(settings);
}

pub(super) fn start(app: AppHandle) -> Result<(), String> {
  tween::start();
  multitouch::start(&app);
  std::thread::Builder::new()
    .name("glide-input".to_owned())
    .spawn(move || run(app))
    .map(|_| ())
    .map_err(|error| format!("Could not start Glide input monitoring: {error}"))
}

fn run(app: AppHandle) {
  let state = session::shared_state();
  let event_app = app.clone();
  let event_state = state.clone();
  let poll_app = app;
  let poll_state = state;
  let result = CGEventTap::with_enabled(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::Default,
    vec![
      CGEventType::MouseMoved,
      CGEventType::OtherMouseDragged,
      CGEventType::ScrollWheel,
      CGEventType::FlagsChanged,
      CGEventType::KeyDown,
      CGEventType::KeyUp,
      CGEventType::LeftMouseDown,
      CGEventType::LeftMouseUp,
      CGEventType::OtherMouseDown,
      CGEventType::OtherMouseUp,
    ],
    move |_, event_type, event| handle_event(&event_app, &event_state, event_type, event),
    || loop {
      unsafe {
        CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, POLL_INTERVAL, false);
      }
      settle_detector(&poll_app, &poll_state);
    },
  );
  if result.is_err() {
    eprintln!("Could not start Glide input monitoring; Accessibility access is required");
  }
}

fn handle_event(
  app: &AppHandle,
  state: &SharedState,
  event_type: CGEventType,
  event: &CGEvent,
) -> CallbackResult {
  native_settings::observe(event_type, event);
  if matches!(event_type, CGEventType::KeyDown | CGEventType::KeyUp) {
    return control_keys::handle_key(app, state, event_type, event);
  }
  if matches!(
    event_type,
    CGEventType::OtherMouseDown | CGEventType::OtherMouseUp
  ) {
    return control_keys::handle_mouse_button(app, state, event_type, event);
  }
  match event_type {
    CGEventType::ScrollWheel => handle_scroll(app, state, event),
    CGEventType::MouseMoved | CGEventType::OtherMouseDragged => handle_mouse(app, state, event),
    CGEventType::FlagsChanged => control_keys::handle_flags_changed(app, state),
    CGEventType::LeftMouseDown => handle_mouse_down(app, state, event),
    CGEventType::LeftMouseUp => handle_mouse_up(state),
    _ => CallbackResult::Keep,
  }
}

fn handle_scroll(app: &AppHandle, state: &SharedState, event: &CGEvent) -> CallbackResult {
  if !is_active(state) {
    let continuous = event.get_integer_value_field(SCROLL_IS_CONTINUOUS_FIELD);
    // A flick keeps emitting momentum scrolls after the fingers lift, all of
    // them located at the anchor the session was pinned to. They must never
    // start a session, and the tail of the gesture that just ended is
    // swallowed so the preview does not flash back into view.
    if event.get_integer_value_field(SCROLL_MOMENTUM_PHASE_FIELD) != 0 {
      if continuous != 0 && is_suppressing_momentum(state) {
        return CallbackResult::Drop;
      }
      return CallbackResult::Keep;
    }
    set_momentum_suppression(state, false);
    if continuous == 0 {
      return CallbackResult::Keep;
    }
    // The gesture Esc cancelled is still under the fingers. Its remainder is
    // swallowed so it neither restarts a session nor leaks scrolls, and the
    // phase that ends it hands the tail over to the momentum suppression.
    if is_suppressing(state, InputKind::Trackpad) {
      if event.get_integer_value_field(SCROLL_PHASE_FIELD)
        & (SCROLL_PHASE_ENDED | SCROLL_PHASE_CANCELLED)
        != 0
      {
        set_suppression(state, InputKind::Trackpad, false);
        set_momentum_suppression(state, true);
      }
      return CallbackResult::Drop;
    }
  }
  if !is_active(state) && !begin_if_titlebar(app, state, InputKind::Trackpad, event.location()) {
    return CallbackResult::Keep;
  }
  if active_input(state) != Some(InputKind::Trackpad) {
    return CallbackResult::Keep;
  }

  // Point deltas already follow physical finger travel, including when natural
  // scrolling is enabled. Glide should feel spatial, not like content scroll.
  let delta_x =
    event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2) as f64;
  let delta_y =
    event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1) as f64;
  if delta_x != 0.0 || delta_y != 0.0 {
    update_detector(app, state, delta_x, delta_y, is_thirds(event));
  }

  let phase = event.get_integer_value_field(SCROLL_PHASE_FIELD);
  if phase & (SCROLL_PHASE_ENDED | SCROLL_PHASE_CANCELLED) != 0 {
    set_momentum_suppression(state, true);
    end_session(app, state, false);
  }
  CallbackResult::Drop
}

fn handle_mouse(app: &AppHandle, state: &SharedState, event: &CGEvent) -> CallbackResult {
  let modifier_down = native_settings::is_down(native_settings::snapshot().mouse_modifier);
  if active_input(state) == Some(InputKind::Mouse) && !modifier_down {
    end_session(app, state, false);
    return CallbackResult::Keep;
  }
  if !is_active(state) && modifier_down && is_suppressing(state, InputKind::Mouse) {
    // Esc cancelled this glide; the modifier has to be released before the next
    // one.
    return CallbackResult::Keep;
  }
  if !is_active(state)
    && modifier_down
    && !begin_if_titlebar(app, state, InputKind::Mouse, event.location())
  {
    return CallbackResult::Keep;
  }
  if active_input(state) == Some(InputKind::Trackpad) {
    // Reaching for the pointer during a trackpad session is a dismissal. The
    // cursor is still pinned at the anchor here, so it stays where it is.
    let delta_x = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64;
    let delta_y = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64;
    let travel = accumulate_pointer_travel(state, delta_x.abs() + delta_y.abs());
    if travel >= POINTER_DISMISS_DISTANCE {
      end_session(app, state, false);
    }
    return CallbackResult::Keep;
  }
  if active_input(state) != Some(InputKind::Mouse) {
    return CallbackResult::Keep;
  }

  if let Some(anchor) = session_anchor(state) {
    event.set_location(anchor);
  }
  let delta_x = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64;
  let delta_y = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64;
  if delta_x != 0.0 || delta_y != 0.0 {
    update_detector(app, state, delta_x, delta_y, is_thirds(event));
  }
  CallbackResult::Drop
}

/// The Glide mouse control and a double click on a titlebar center the window the
/// same way a trackpad double tap does. Both events of the second click are
/// dropped, so the application below never sees the double click it would zoom
/// on; every other press, single clicks included, passes straight through.
fn handle_mouse_down(app: &AppHandle, state: &SharedState, event: &CGEvent) -> CallbackResult {
  let settings = native_settings::snapshot();
  // The double tap and this click are one action under two inputs, so the one
  // setting turns both of them off.
  if !settings.enabled || !settings.double_tap_center {
    return CallbackResult::Keep;
  }
  if !native_settings::is_down(settings.mouse_modifier)
    || event.get_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE) != DOUBLE_CLICK_STATE
  {
    return CallbackResult::Keep;
  }
  let point = event.location();
  // One of ours answers natively, a foreign one through Accessibility; either
  // titlebar is a titlebar as far as this click is concerned.
  if !any_titlebar(app, point) {
    return CallbackResult::Keep;
  }
  // An internal takeover, not a commit: minimizing the window this click is
  // about to center would be two destinations at once. The cancel's restore, if
  // the click landed on a glide that had already moved something, is preempted
  // by the centering below before the tween thread can take a single step.
  end_session(app, state, true);
  set_mouse_up_swallow(state, true);
  center_window_at(app, point);
  CallbackResult::Drop
}

fn handle_mouse_up(state: &SharedState) -> CallbackResult {
  if take_mouse_up_swallow(state) {
    CallbackResult::Drop
  } else {
    CallbackResult::Keep
  }
}

fn is_thirds(event: &CGEvent) -> bool {
  let _ = event;
  native_settings::is_down(native_settings::snapshot().thirds_modifier)
}

pub(super) fn supports_control(control: crate::glide::settings::GlideControl) -> bool {
  control::NativeControl::from_control(control).is_some()
}
