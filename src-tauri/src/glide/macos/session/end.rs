// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commit, cancellation, cursor landing, and preview-fade teardown.

use tauri::AppHandle;

use super::{Session, SharedState, STATE};
use crate::glide::platform::{
  cursor::{landing_point, release_cursor},
  native_settings,
  tween::animate_to,
};
use crate::glide::{finish, finish_with_fade};

/// Ends the live session, if there is one. `cancelled` rides out with the end
/// so the detector knows whether to commit what it had armed.
pub(in crate::glide::platform) fn end_session(
  app: &AppHandle,
  state: &SharedState,
  cancelled: bool,
) {
  let minimize = state.lock().is_ok_and(|state| {
    state
      .session
      .as_ref()
      .is_some_and(|session| session.runtime.should_minimize(cancelled))
  });
  // Only a revealed glide breaks a double tap in the making. A tap sheds a few
  // scroll events of its own, and the invisible session they open must not
  // clear the candidate that same tap just stored.
  let Some(mut session) = take_session(app, state, cancelled) else {
    return;
  };
  if minimize {
    session.target.minimize();
  }
  if session.revealed {
    if let Ok(mut state) = state.lock() {
      state.tap_candidate = None;
    }
  }
}

fn take_session(app: &AppHandle, state: &SharedState, cancelled: bool) -> Option<Session> {
  let session = state
    .lock()
    .ok()
    .and_then(|mut state| state.session.take())?;
  // A cancel undoes the whole gesture: the window animates back to the frame
  // the session captured, out of wherever the last transition left it. A commit
  // stops nothing, so the tween that is still arriving finishes on its own.
  if cancelled && session.moved {
    animate_to(&session.target, session.original_frame, None);
  }
  if cancelled || !session.revealed {
    release_cursor(session.anchor, session.revealed);
    finish(app, session.anchor.x, session.anchor.y, cancelled);
    return Some(session);
  }

  if let Ok(mut state) = state.lock() {
    state.fading = true;
  }
  let anchor = session.anchor;
  let moved = session.moved;
  let grip = session.target.duplicate();
  let original = session.original_frame;
  let cursor_follows = native_settings::snapshot().cursor_follows
    && !session.runtime.commits_terminal_action(cancelled);
  let returned_to_origin = session.returned_to_origin;
  if returned_to_origin {
    // Nothing has to chase the window, so restore association immediately;
    // a delayed warp would pull back the user's next physical mouse move.
    release_cursor(anchor, true);
  }
  finish_with_fade(
    app,
    anchor.x,
    anchor.y,
    Box::new(move || {
      if returned_to_origin {
        return;
      }
      let landing = if moved && cursor_follows {
        grip
          .frame()
          .map(|achieved| landing_point(anchor, original, achieved))
          .unwrap_or(anchor)
      } else {
        anchor
      };
      release_cursor(landing, true);
    }),
    Box::new(move || {
      if let Some(state) = STATE.get() {
        if let Ok(mut state) = state.lock() {
          state.fading = false;
        }
      }
    }),
  );
  Some(session)
}
