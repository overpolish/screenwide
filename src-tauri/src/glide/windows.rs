// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicBool, Ordering},
  OnceLock,
};

use tauri::AppHandle;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
#[path = "windows/center.rs"]
mod center;
#[path = "windows/control.rs"]
mod control;
#[path = "windows/cursor.rs"]
mod cursor;
#[path = "windows/input_kind.rs"]
mod input_kind;
#[path = "windows/input_window.rs"]
mod input_window;
#[path = "windows/key.rs"]
mod key;
#[path = "windows/key_hook.rs"]
mod key_hook;
#[path = "windows/keyboard.rs"]
mod keyboard;
#[path = "windows/monitors.rs"]
mod monitors;
#[path = "windows/mouse_motion.rs"]
mod mouse_motion;
#[path = "windows/native_settings.rs"]
mod native_settings;
#[path = "windows/native_trackpad.rs"]
mod native_trackpad;
#[path = "windows/navigation.rs"]
mod navigation;
#[path = "windows/precision_touchpad.rs"]
mod precision_touchpad;
#[path = "windows/preview_windows.rs"]
mod preview_windows;
#[path = "windows/raw_input.rs"]
mod raw_input;
#[path = "windows/release_tracker.rs"]
mod release_tracker;
#[path = "windows/session.rs"]
mod session;

#[path = "windows/spaces.rs"]
mod spaces;
#[path = "windows/taps.rs"]
mod taps;
#[path = "windows/target.rs"]
mod target;
#[path = "windows/titlebar.rs"]
mod titlebar;
#[path = "windows/trackpad.rs"]
mod trackpad;
#[path = "windows/tween.rs"]
mod tween;
#[path = "windows/virtual_desktops.rs"]
mod virtual_desktops;
#[path = "windows/wheel_hook.rs"]
mod wheel_hook;

use input_kind::InputKind;

static APP: OnceLock<AppHandle> = OnceLock::new();
/// Real pointer travel during a trackpad session commits the gesture.
const POINTER_DISMISS_DISTANCE: f64 = 3.0;
static WHEEL_HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(super) fn start(app: AppHandle) -> Result<(), String> {
  let _ = APP.set(app.clone());
  native_settings::trace_state("startup");
  preview_windows::preload(&app);
  titlebar::cache_own_windows(&app);
  tween::start();
  let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
  std::thread::Builder::new()
    .name("glide-input".to_owned())
    .spawn(move || input_window::run(app, ready_tx))
    .map_err(|error| format!("Could not start Glide input monitoring: {error}"))?;
  ready_rx
    .recv()
    .map_err(|_| "Glide input monitoring stopped before it was ready".to_owned())?
}

pub(super) fn apply_settings(settings: &crate::glide::settings::GlideSettings) {
  native_settings::apply(settings);
  native_settings::trace_state("settings-applied");
  if !settings.enabled {
    finish_current_session(true);
  }
}

pub(super) use raw_input::handle_raw_input;

pub(super) fn set_icon(session_id: u64, path: Option<std::path::PathBuf>) {
  session::set_icon(session_id, path.clone());
  spaces::set_icon(session_id, path.clone());
  if let Some(app) = APP.get() {
    preview_windows::set_icon(app, session_id, path);
  }
}

pub(super) fn set_wheel_hook_active(active: bool) {
  WHEEL_HOOK_ACTIVE.store(active, Ordering::Relaxed);
}

pub(super) use mouse_motion::handle_mouse_delta;

fn begin_session(input: InputKind) -> bool {
  let Some(app) = APP.get() else {
    return false;
  };
  session::begin(app, input)
}

pub(super) fn tick() {
  if let Some(app) = APP.get() {
    let settings = native_settings::snapshot();
    spaces::clear_committed_if_released();
    session::clear_monitor_commit_if_released();
    if navigation::cancelled()
      && !native_settings::is_down(settings.mouse_modifier)
      && !native_settings::is_down(settings.spaces_modifier)
      && !native_settings::is_down(settings.monitors_modifier)
    {
      crate::glide::core::trace::input("windows-input", "clear-cancel-after-release");
      navigation::clear_cancel();
    }
    if spaces::active_input().is_none()
      && native_settings::is_down(settings.spaces_modifier)
      && !native_trackpad::pointer_episode_active()
    {
      let mut point = windows::Win32::Foundation::POINT::default();
      if unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.is_ok() {
        if let Some((target, _)) = target::WindowTarget::at(app, point) {
          spaces::arm(app, target, point);
        }
      }
    }
    if spaces::active_input().is_some() {
      if !settings.enabled {
        spaces::end(app, true);
        return;
      }
      if !native_settings::is_down(settings.spaces_modifier) {
        spaces::end(app, false);
        return;
      }
      spaces::poll(app);
      return;
    }
    if session::active_input().is_none()
      && native_settings::is_down(settings.monitors_modifier)
      && !native_trackpad::pointer_episode_active()
      && !navigation::cancelled()
    {
      if begin_session(InputKind::TrackpadScroll) {
        session::arm_monitor(app);
      }
    }
    if session::monitor_mode() && !native_settings::is_down(settings.monitors_modifier) {
      session::end(app, false);
      return;
    }
    if mouse_session_released(
      session::active_input(),
      if session::monitor_mode() {
        native_settings::is_down(settings.monitors_modifier)
      } else {
        native_settings::is_down(settings.mouse_modifier)
      },
    ) {
      session::end(app, false);
      return;
    }
    if !session::monitor_mode()
      && session::pointer_displacement()
        .is_some_and(|distance| distance >= POINTER_DISMISS_DISTANCE)
    {
      session::end(app, false);
      return;
    }
    session::tick(app);
  }
}

fn mouse_session_released(active: Option<InputKind>, modifier_down: bool) -> bool {
  active == Some(InputKind::Mouse) && !modifier_down
}

fn finish_current_session(cancelled: bool) {
  if let Some(app) = APP.get() {
    if spaces::active_input().is_some() {
      spaces::end(app, cancelled);
      return;
    }
    session::end(app, cancelled);
  }
}

pub(super) fn end_session(app: &AppHandle) {
  if spaces::active_input().is_some() {
    spaces::end(app, true);
    return;
  }
  session::end(app, true);
}

pub(super) use taps::{clear_trackpad_tap_candidate, register_trackpad_tap};

pub(super) fn suspend_for_capture(app: &AppHandle) {
  navigation::cancel_until_release();
  spaces::cancel(app);
  session::end(app, true);
}

pub(super) fn supports_control(control: crate::glide::settings::GlideControl) -> bool {
  control::NativeControl::from_control(control).is_some()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tick_ends_only_a_mouse_session_whose_control_was_released() {
    assert!(mouse_session_released(Some(InputKind::Mouse), false));
    assert!(!mouse_session_released(Some(InputKind::Mouse), true));
    assert!(!mouse_session_released(
      Some(InputKind::TrackpadContacts),
      false
    ));
    assert!(!mouse_session_released(None, false));
  }
}
