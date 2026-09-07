// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small, lock-scoped reads and updates used by the macOS event adapter.

use cidre::cg;
use core_graphics::geometry::CGPoint;

use super::{InputKind, SharedState};

pub(in crate::glide::platform) fn session_anchor(state: &SharedState) -> Option<CGPoint> {
  state
    .lock()
    .ok()
    .and_then(|state| state.session.as_ref().map(|session| session.anchor))
}

pub(in crate::glide::platform) fn accumulate_pointer_travel(
  state: &SharedState,
  distance: f64,
) -> f64 {
  state
    .lock()
    .ok()
    .and_then(|mut state| {
      state.session.as_mut().map(|session| {
        session.pointer_travel += distance;
        session.pointer_travel
      })
    })
    .unwrap_or(0.0)
}

pub(in crate::glide::platform) fn active_input(state: &SharedState) -> Option<InputKind> {
  state
    .lock()
    .ok()
    .and_then(|state| state.session.as_ref().map(|session| session.input))
}

pub(in crate::glide::platform) fn is_active(state: &SharedState) -> bool {
  active_input(state).is_some()
}

pub(in crate::glide::platform) fn monitor_mode(state: &SharedState) -> bool {
  state.lock().ok().is_some_and(|state| {
    state
      .session
      .as_ref()
      .is_some_and(|session| session.monitors.is_some())
  })
}

pub(in crate::glide::platform) fn session_id(state: &SharedState) -> Option<u64> {
  state
    .lock()
    .ok()
    .and_then(|state| state.session.as_ref().map(|session| session.id))
}

pub(in crate::glide::platform) fn mouse_center_context(
  state: &SharedState,
) -> Option<(
  super::super::tween::WindowTarget,
  cg::Rect,
  (f64, f64),
  (f64, f64),
)> {
  let (target, work_origin, work_size) = state.lock().ok().and_then(|state| {
    let session = state.session.as_ref()?;
    if session.input != InputKind::Mouse || session.monitors.is_some() {
      return None;
    }
    Some((
      session.target.duplicate(),
      session.work_origin,
      session.work_size,
    ))
  })?;
  let frame = target.frame()?;
  Some((target, frame, work_origin, work_size))
}
