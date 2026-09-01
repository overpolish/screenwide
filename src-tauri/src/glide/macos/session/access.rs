// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small, lock-scoped reads and updates used by the macOS event adapter.

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
