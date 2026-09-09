// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commit, cancellation, cursor landing, and preview-fade teardown.

use tauri::AppHandle;

use super::{Session, SharedState, STATE};
use crate::glide::platform::{
  cursor::{landing_point, release_cursor},
  native_settings,
  tween::{after_current, animate_to},
};
use crate::glide::{finish, finish_with_fade};

/// Ends the live session, if there is one. `cancelled` rides out with the end
/// so the detector knows whether to commit what it had armed.
pub(in crate::glide::platform) fn end_session(
  app: &AppHandle,
  state: &SharedState,
  cancelled: bool,
) {
  if !cancelled {
    super::detector::finish_opening(app, state);
  }
  let completion = crate::glide::core::activity::BusyLease::acquire();
  let minimize = state.lock().is_ok_and(|state| {
    state
      .session
      .as_ref()
      .is_some_and(|session| session.runtime.should_minimize(cancelled))
  });
  // Only a revealed glide breaks a double tap in the making. A tap sheds a few
  // scroll events of its own, and the invisible session they open must not
  // clear the candidate that same tap just stored.
  let Some(mut session) = take_session(app, state, cancelled, completion) else {
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

fn take_session(
  app: &AppHandle,
  state: &SharedState,
  cancelled: bool,
  completion: crate::glide::core::activity::BusyLease,
) -> Option<Session> {
  let mut session = state.lock().ok().and_then(|mut state| {
    let session = state.session.take()?;
    if !cancelled && session.revealed {
      state.fading = Some(session.id);
    }
    Some(session)
  })?;
  let monitor_selection = session
    .monitors
    .as_ref()
    .is_some_and(|selection| selection.active);
  if monitor_selection {
    crate::glide::platform::spaces::preview_windows::dismiss(app, cancelled);
    if !cancelled {
      super::monitors::commit(app, &mut session);
    }
  }
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

  let anchor = session.anchor;
  let moved = session.moved;
  let grip = session.target.duplicate();
  let original = session.original_frame;
  let monitor_landing = super::monitors::landing(&session);
  let cursor_follows = native_settings::snapshot().cursor_follows
    && !session.runtime.commits_terminal_action(cancelled);
  let cursor_completion = completion.clone();
  // A monitor release that selected its source, or lost its destination
  // during revalidation, performed no move and must release the cursor now.
  let returned_to_origin =
    releases_cursor_immediately(monitor_selection, session.moved, session.returned_to_origin);
  if returned_to_origin {
    // Nothing has to chase the window, so restore association immediately;
    // a delayed warp would pull back the user's next physical mouse move.
    release_cursor(anchor, true);
  }
  let app = app.clone();
  // A quick lift can start the move itself. Read the cursor's destination only
  // after that tween and its fit correction finish, keeping the preview alive.
  after_current(Box::new(move || {
    finish_with_fade(
      &app,
      anchor.x,
      anchor.y,
      Box::new(move || {
        let _completion = cursor_completion;
        if returned_to_origin {
          return;
        }
        let landing = if moved && cursor_follows {
          monitor_landing.unwrap_or_else(|| {
            grip
              .frame()
              .map(|achieved| landing_point(anchor, original, achieved))
              .unwrap_or(anchor)
          })
        } else {
          anchor
        };
        release_cursor(landing, true);
      }),
      Box::new(move || {
        crate::glide::platform::spaces::preview_windows::after_dismissed(Box::new(|| {
          let _completion = completion;
          if let Some(state) = STATE.get() {
            if let Ok(mut state) = state.lock() {
              state.fading = None;
            }
          }
        }));
      }),
    )
  }));
  Some(session)
}

fn releases_cursor_immediately(monitor_selection: bool, moved: bool, returned: bool) -> bool {
  monitor_selection && (!moved || returned)
}

#[cfg(test)]
mod tests {
  use super::releases_cursor_immediately;

  #[test]
  fn monitor_noop_release_does_not_defer_cursor_release() {
    assert!(releases_cursor_immediately(true, false, false));
    assert!(releases_cursor_immediately(true, true, true));
    assert!(!releases_cursor_immediately(false, false, false));
  }

  #[test]
  fn flick_into_occupied_slot_keeps_cursor_hidden_through_completion() {
    assert!(!releases_cursor_immediately(false, true, true));
    assert!(!releases_cursor_immediately(false, true, false));
  }
}
