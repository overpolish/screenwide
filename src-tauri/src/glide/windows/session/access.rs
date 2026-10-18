// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the input hooks read of the live session, and the monitor commit
//! latch.

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

static MONITOR_COMMITTED_UNTIL_RELEASE: AtomicBool = AtomicBool::new(false);

fn with_session<R>(f: impl FnOnce(&mut WindowsSession) -> R) -> Option<R> {
  shared::with_session::<WindowsGlide, R>(f)
}

pub(in crate::glide::windows) fn next_id() -> u64 {
  shared::next_id()
}

pub(in crate::glide::windows) fn promote_scroll_to_contacts() -> bool {
  with_session(|session| {
    if session.input != InputKind::TrackpadScroll {
      return false;
    }
    session.extra.armed = false;
    session.input = InputKind::TrackpadContacts;
    true
  })
  .unwrap_or(false)
}

pub(super) fn monitor_commit_latched() -> bool {
  MONITOR_COMMITTED_UNTIL_RELEASE.load(Ordering::Acquire)
}

pub(super) fn latch_monitor_commit() {
  MONITOR_COMMITTED_UNTIL_RELEASE.store(true, Ordering::Release);
}

pub(in crate::glide::windows) fn clear_monitor_commit_if_released() {
  let settings = native_settings::snapshot();
  if !native_settings::is_down(settings.monitors_modifier) {
    MONITOR_COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
  }
}

pub(in crate::glide::windows) fn clear_monitor_commit() {
  MONITOR_COMMITTED_UNTIL_RELEASE.store(false, Ordering::Release);
}

pub(in crate::glide::windows) fn active_input() -> Option<InputKind> {
  with_session(|session| session.input)
}

pub(in crate::glide::windows) fn revealed() -> bool {
  with_session(|session| session.revealed).unwrap_or(false)
}

pub(in crate::glide::windows) fn target_hwnd() -> Option<isize> {
  with_session(|session| session.target.native_hwnd().0 as isize)
}

pub(in crate::glide::windows) fn target() -> Option<WindowTarget> {
  with_session(|session| (session.input == InputKind::Mouse).then_some(session.target)).flatten()
}

pub(in crate::glide::windows) fn set_icon(id: u64, path: Option<PathBuf>) {
  with_session(|session| {
    if session.id == id {
      session.extra.icon_path = path;
    }
  });
}

pub(in crate::glide::windows) fn monitor_mode() -> bool {
  with_session(|session| session.monitors.is_some()).unwrap_or(false)
}

pub(in crate::glide::windows) fn anchor() -> Option<POINT> {
  with_session(|session| point(session.anchor))
}

pub(in crate::glide::windows) fn pointer_displacement() -> Option<f64> {
  let anchor =
    with_session(|session| session.input.is_trackpad().then(|| point(session.anchor))).flatten()?;
  let pointer = cursor_position()?;
  Some(f64::from(pointer.x.abs_diff(anchor.x)) + f64::from(pointer.y.abs_diff(anchor.y)))
}

pub(in crate::glide::windows) fn accumulate_pointer_travel(distance: f64) -> f64 {
  with_session(|session| {
    session.pointer_travel += distance;
    session.pointer_travel
  })
  .unwrap_or(0.0)
}

pub(crate) fn promote_armed_to_mouse() -> bool {
  with_session(|session| {
    if session.extra.armed && session.input == InputKind::TrackpadScroll {
      session.extra.armed = false;
      session.input = InputKind::Mouse;
      true
    } else {
      false
    }
  })
  .unwrap_or(false)
}
