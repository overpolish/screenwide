// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two-finger double tap that centers a window: the pairing of one tap with
//! the one before it, and nothing else. The taps themselves are recognised by
//! the multitouch monitor; what makes a pair of them is here.

use std::time::{Duration, Instant};

use core_graphics::geometry::CGPoint;
use tauri::AppHandle;

use super::super::{center::center_window_at, native_settings, own_window::any_titlebar};
use super::STATE;

/// How long a first tap waits for its partner.
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(350);
/// How far apart, in logical pixels, the two taps of a double tap may land.
const TAP_RADIUS: f64 = 20.0;

/// Records a two-finger tap the multitouch monitor recognised, at the cursor it
/// happened under. The second tap close behind the first, and close to it on
/// screen, centers the window under it; a glide in between breaks the pair.
pub fn register_tap(app: &AppHandle, point: CGPoint) {
  let settings = native_settings::snapshot();
  if crate::capture_overlays::blocks_glide(app) || !settings.enabled || !settings.double_tap_center
  {
    return;
  }
  let Some(state) = STATE.get() else {
    return;
  };
  // A tap during a revealed glide belongs to the glide. An unrevealed session
  // is usually this very tap seen through its own scroll events - the lift can
  // reach the multitouch monitor before the scroll phase ends it - so it does
  // not block. Only a titlebar tap has a window to center - one of ours or a
  // foreign one - and both taps of a pair have to land on one.
  let mid_glide = state.lock().is_ok_and(|state| {
    state
      .session
      .as_ref()
      .is_some_and(|session| session.revealed)
  });
  if mid_glide || !any_titlebar(app, point) {
    return;
  }

  let now = Instant::now();
  let is_double_tap = state.lock().is_ok_and(|mut state| {
    let paired = state.tap_candidate.take().is_some_and(|(at, previous)| {
      now.duration_since(at) <= DOUBLE_TAP_WINDOW
        && (previous.x - point.x).hypot(previous.y - point.y) <= TAP_RADIUS
    });
    if !paired {
      state.tap_candidate = Some((now, point));
    }
    paired
  });

  if is_double_tap {
    center_window_at(app, point);
  }
}
