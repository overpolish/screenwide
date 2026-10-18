// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commit, cancellation, pointer landing and the preview's fade.

use tauri::AppHandle;

use super::{finish_opening, GlidePlatform, Point};
use crate::glide::core::tween::{self, GlideWindow};
use crate::glide::core::tween_completion::after_current;
use crate::glide::core::{frames_match, landing_point, GlideFrame};
use crate::glide::{finish, finish_with_fade, BusyLease};

/// Ends the live session, if there is one. `cancelled` rides out with the
/// end so the detector knows whether to commit what it had armed, and
/// `cursor_follows` is the "Move pointer with window" setting.
pub(crate) fn end<P: GlidePlatform>(app: &AppHandle, cancelled: bool, cursor_follows: bool) {
  if !cancelled {
    finish_opening::<P>(app);
  }
  let completion = BusyLease::acquire();
  let Some(mut session) = P::with_state(|state| {
    let session = P::slot(state).take()?;
    P::session_taken(state, &session, cancelled);
    Some(session)
  })
  .flatten() else {
    return;
  };
  let monitor_selection = P::end_monitors(app, &mut session, cancelled);
  // A cancel undoes the whole gesture: the window goes back to the frame the
  // session captured, out of wherever the last transition left it. A commit
  // stops nothing, so the tween still arriving finishes on its own.
  if cancelled && session.moved {
    session.target.restore(session.original_frame);
  }
  if session.runtime.should_minimize(cancelled) {
    // A late frame of the tween must not fight the minimize.
    tween::cancel();
    session.target.minimize();
  }
  // A monitor release that selected its source, or lost its destination,
  // performed no move: the pointer goes back now, since a delayed warp would
  // pull back the hand's next move.
  let immediate =
    releases_pointer_immediately(monitor_selection, session.moved, session.returned_to_origin);
  if cancelled || !session.revealed {
    if immediate && session.revealed {
      P::land_pointer(session.anchor);
    } else {
      P::release_pointer(session.anchor, session.revealed);
    }
    finish(app, session.anchor.0, session.anchor.1, cancelled);
    return;
  }
  if immediate {
    P::land_pointer(session.anchor);
  }
  let landing = Landing::<P> {
    anchor: session.anchor,
    original: session.original_frame,
    grip: session.target.duplicate(),
    follows: cursor_follows && session.moved && !session.runtime.commits_terminal_action(false),
    immediate,
  };
  let id = session.id;
  let app = app.clone();
  // A quick lift can start the move itself, and the committed preview stays
  // up through the final tween and its fit correction: the pointer's landing
  // is read only once they finish, from the frame the window reached.
  after_current(Box::new(move || {
    let cursor_completion = completion.clone();
    let anchor = landing.anchor;
    finish_with_fade(
      &app.clone(),
      anchor.0,
      anchor.1,
      Box::new(move || {
        let _completion = cursor_completion;
        landing.land();
      }),
      Box::new(move || P::after_fade(&app, id, completion)),
    );
  }));
}

/// Everything the pointer's landing needs once the window has arrived.
struct Landing<P: GlidePlatform> {
  anchor: Point,
  original: GlideFrame,
  grip: P::Window,
  follows: bool,
  immediate: bool,
}

impl<P: GlidePlatform> Landing<P> {
  fn land(self) {
    if self.immediate {
      return;
    }
    let reached = self
      .follows
      .then(|| self.grip.frame())
      .flatten()
      .filter(|achieved| !frames_match(self.original, *achieved, 1.0))
      .map(|achieved| landing_point(self.anchor, self.original, achieved));
    match reached {
      Some(point) => P::land_pointer(point),
      None => P::release_pointer(self.anchor, true),
    }
  }
}

fn releases_pointer_immediately(monitor_selection: bool, moved: bool, returned: bool) -> bool {
  monitor_selection && (!moved || returned)
}

#[cfg(test)]
mod tests {
  use super::releases_pointer_immediately;

  #[test]
  fn a_monitor_release_that_moved_nothing_lets_go_of_the_pointer_at_once() {
    assert!(releases_pointer_immediately(true, false, false));
    assert!(releases_pointer_immediately(true, true, true));
    assert!(!releases_pointer_immediately(true, true, false));
    assert!(!releases_pointer_immediately(false, false, false));
  }
}
