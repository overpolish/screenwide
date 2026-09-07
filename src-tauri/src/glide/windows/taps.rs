// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::input_kind::InputKind;
use super::{native_settings, session, spaces, titlebar, wheel_hook, APP};
use std::sync::Mutex;
use std::time::{Duration, Instant};
static LAST_TRACKPAD_TAP: Mutex<Option<(Instant, i32, i32, isize)>> = Mutex::new(None);

pub(crate) fn register_trackpad_tap() {
  let settings = native_settings::snapshot();
  let Some(app) = APP.get() else {
    return;
  };
  if !settings.enabled
    || !settings.double_tap_center
    || native_settings::is_down(settings.monitors_modifier)
    || native_settings::is_down(settings.spaces_modifier)
    || spaces::active_input().is_some()
    || spaces::is_closing()
    || session::revealed()
  {
    return;
  }
  let mut point = windows::Win32::Foundation::POINT::default();
  if unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.is_err()
    || crate::capture_overlays::blocks_glide(app)
  {
    clear_trackpad_tap_candidate();
    return;
  }
  let Some(hwnd) = titlebar::cached_window_at(point).map(|hwnd| hwnd.0 as isize) else {
    clear_trackpad_tap_candidate();
    return;
  };
  if !super::center::allowed(app, InputKind::TrackpadContacts) {
    clear_trackpad_tap_candidate();
    return;
  }
  let now = Instant::now();
  let paired = LAST_TRACKPAD_TAP.lock().ok().is_some_and(|mut last| {
    let paired = last.as_ref().is_some_and(|(at, x, y, previous_hwnd)| {
      now.duration_since(*at) <= Duration::from_millis(350)
        && (point.x - x).unsigned_abs() <= 20
        && (point.y - y).unsigned_abs() <= 20
        && *previous_hwnd == hwnd
    });
    *last = if paired {
      None
    } else {
      Some((now, point.x, point.y, hwnd))
    };
    paired
  });
  if paired {
    let _ = wheel_hook::queue_center(hwnd, (point.x, point.y), InputKind::TrackpadContacts);
  }
}

pub(crate) fn clear_trackpad_tap_candidate() {
  if let Ok(mut last) = LAST_TRACKPAD_TAP.lock() {
    *last = None;
  }
}
