// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The latches the monitor keeps outside a live session: what is left of a
//! gesture Esc cancelled, the momentum that outlives a scroll, and the mouse
//! release an intercepted double click still owes its application.

use super::{InputKind, SharedState};

/// Marks the input a cancelled session was driven by, so the rest of the
/// gesture behind Esc cannot open a new one, and clears it again once that
/// gesture is over.
pub fn set_suppression(state: &SharedState, input: InputKind, suppress: bool) {
  if let Ok(mut state) = state.lock() {
    match input {
      InputKind::Trackpad => state.suppress_gesture = suppress,
      InputKind::Mouse => state.suppress_mouse = suppress,
    }
  }
}

pub fn is_suppressing(state: &SharedState, input: InputKind) -> bool {
  state.lock().ok().is_some_and(|state| match input {
    InputKind::Trackpad => state.suppress_gesture,
    InputKind::Mouse => state.suppress_mouse,
  })
}

pub fn set_mouse_up_swallow(state: &SharedState, swallow: bool) {
  if let Ok(mut state) = state.lock() {
    state.swallow_mouse_up = swallow;
  }
}

pub fn take_mouse_up_swallow(state: &SharedState) -> bool {
  state
    .lock()
    .ok()
    .is_some_and(|mut state| std::mem::take(&mut state.swallow_mouse_up))
}

pub fn set_momentum_suppression(state: &SharedState, suppress: bool) {
  if let Ok(mut state) = state.lock() {
    state.suppress_momentum = suppress;
  }
}

pub fn is_suppressing_momentum(state: &SharedState) -> bool {
  state
    .lock()
    .ok()
    .map(|state| state.suppress_momentum)
    .unwrap_or(false)
}
