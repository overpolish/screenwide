// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

static MONITOR_COMMITTED_UNTIL_RELEASE: AtomicBool = AtomicBool::new(false);

pub(in crate::glide::platform) fn next_id() -> u64 {
  NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
}

pub(in crate::glide::platform) fn promote_scroll_to_contacts() -> bool {
  STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut() else {
      return false;
    };
    if session.input != InputKind::TrackpadScroll {
      return false;
    }
    session.armed = false;
    session.input = InputKind::TrackpadContacts;
    true
  })
}

pub(super) fn monitor_commit_latched() -> bool {
  MONITOR_COMMITTED_UNTIL_RELEASE.load(Ordering::Acquire)
}

pub(super) fn latch_monitor_commit() {
  MONITOR_COMMITTED_UNTIL_RELEASE.store(true, Ordering::Release);
}

pub(in crate::glide::platform) fn clear_monitor_commit_if_released() {
  let settings = native_settings::snapshot();
  if !native_settings::is_down(settings.monitors_modifier) {
    MONITOR_COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
  }
}

pub(in crate::glide::platform) fn clear_monitor_commit() {
  MONITOR_COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
}

pub(in crate::glide::platform) fn active_input() -> Option<InputKind> {
  STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.input))
}

pub(in crate::glide::platform) fn revealed() -> bool {
  STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.revealed))
    .unwrap_or(false)
}

pub(in crate::glide::platform) fn target_hwnd() -> Option<isize> {
  STATE.lock().ok().and_then(|state| {
    state
      .as_ref()
      .map(|session| session.target.native_hwnd().0 as isize)
  })
}

pub(in crate::glide::platform) fn target() -> Option<WindowTarget> {
  STATE.lock().ok().and_then(|state| {
    state
      .as_ref()
      .filter(|session| session.input == InputKind::Mouse)
      .map(|session| session.target)
  })
}

pub(in crate::glide::platform) fn set_icon(id: u64, path: Option<PathBuf>) {
  if let Ok(mut state) = STATE.lock() {
    if let Some(session) = state.as_mut().filter(|session| session.id == id) {
      session.icon_path = path;
    }
  }
}

pub(in crate::glide::platform) fn monitor_mode() -> bool {
  STATE.lock().ok().is_some_and(|state| {
    state
      .as_ref()
      .is_some_and(|session| session.monitors.is_some())
  })
}

pub(in crate::glide::platform) fn anchor() -> Option<POINT> {
  STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.anchor))
}

pub(in crate::glide::platform) fn pointer_displacement() -> Option<f64> {
  let anchor = STATE.lock().ok().and_then(|state| {
    state
      .as_ref()
      .filter(|session| session.input.is_trackpad())
      .map(|session| session.anchor)
  })?;
  let mut pointer = POINT::default();
  unsafe { GetCursorPos(&mut pointer) }.ok()?;
  Some(f64::from(pointer.x.abs_diff(anchor.x)) + f64::from(pointer.y.abs_diff(anchor.y)))
}

pub(in crate::glide::platform) fn accumulate_pointer_travel(distance: f64) -> f64 {
  STATE
    .lock()
    .ok()
    .and_then(|mut state| {
      state.as_mut().map(|session| {
        session.pointer_travel += distance;
        session.pointer_travel
      })
    })
    .unwrap_or(0.0)
}

pub(crate) fn promote_armed_to_mouse() -> bool {
  STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut() else {
      return false;
    };
    if session.armed && session.input == InputKind::TrackpadScroll {
      session.armed = false;
      session.input = InputKind::Mouse;
      true
    } else {
      false
    }
  })
}
