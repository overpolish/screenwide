// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicBool, Ordering},
  OnceLock,
};

use tauri::AppHandle;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
mod center;
mod control;
mod cursor;
mod input_kind;
mod input_window;
mod key;
mod key_hook;
mod keyboard;
mod monitors;
mod mouse_motion;
mod native_settings;
mod native_trackpad;
mod navigation;
mod precision_touchpad;
mod preview_windows;
mod raw_input;
mod release_tracker;
mod session;

mod spaces;
mod taps;
mod target;
mod titlebar;
mod trackpad;
mod tween;
mod virtual_desktops;
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
      && begin_session(InputKind::TrackpadScroll)
    {
      session::arm_monitor(app);
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
