// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two commands a live session serves while it is still running: the
//! deferred reveal of the preview, and each region the detector recognises as
//! the gesture travels. Both are guarded by the session id - a command that
//! arrives after its session ended belongs to nobody - and both hand their work
//! off from outside the state lock the event tap is also waiting on.

use cidre::cg;
use tauri::AppHandle;

use super::super::tween::{animate_to, FitContext};
use super::{Session, STATE};
use crate::glide::region_rect::{region_gravity, region_rect, PlacedRegion};

/// Shows the preview once the detector has a destination to draw. Until then a
/// session is invisible, so a stray scroll over a titlebar neither flashes an
/// empty window nor takes the cursor away.
pub fn reveal(app: &AppHandle, session_id: u64) -> Result<(), String> {
  let state = STATE
    .get()
    .ok_or_else(|| "Glide input monitoring is not running".to_owned())?;
  let blocks_hover = {
    let mut monitor = state
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut monitor.session, session_id)?;
    if session.revealed {
      return Ok(());
    }
    session.revealed = true;
    // Hidden while the lock is held, so a lift on the tap thread cannot slip
    // between the mark and the hide and leave the cursor hidden for good.
    if let Err(error) = super::hide_cursor() {
      eprintln!("{error}");
    }
    session.input == super::InputKind::Mouse
  };

  let state = state.clone();
  let main_app = app.clone();
  app
    .run_on_main_thread(move || {
      // The session can end between this command arriving and the main-thread
      // hop, and `finish`'s hide is already queued ahead of a late show.
      let active = state.lock().is_ok_and(|monitor| {
        monitor
          .session
          .as_ref()
          .is_some_and(|session| session.id == session_id)
      });
      if active {
        if let Err(error) = crate::windows::show_glide_preview(&main_app, blocks_hover) {
          eprintln!("Could not present Glide: {error}");
        }
      }
    })
    .map_err(|error| error.to_string())
}

/// Moves the session's window into a region of the work area it was anchored
/// on. This is every transition of a live gesture, not the end of one: the
/// window follows the fingers, and the lift has nothing left to place.
pub fn region_moved(app: &AppHandle, session_id: u64, region: &PlacedRegion) -> Result<(), String> {
  let state = STATE
    .get()
    .ok_or_else(|| "Glide input monitoring is not running".to_owned())?;
  // Everything the animation needs leaves the lock with the destination: the
  // Accessibility calls it makes must never run while the event tap is waiting
  // on this same mutex.
  let (target, origin, size, fit) = {
    let mut monitor = state
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut monitor.session, session_id)?;
    // Marked here rather than at the end, so a cancel only has a restore to run
    // when the window was actually taken somewhere.
    session.moved = true;
    let (origin, size) = region_rect(
      session.work_origin,
      session.work_size,
      region,
      // The gap is the tap's snapshot, not the session's: it is read per
      // placement so a change lands on the next transition.
      super::super::native_settings::snapshot().window_gap,
    );
    // Read out with the destination, so the settle can place a constrained
    // window and report the frame it got without touching the session again.
    let fit = FitContext {
      app: app.clone(),
      session_id,
      gravity: region_gravity(region),
      work_origin: session.work_origin,
      work_size: session.work_size,
    };
    (session.target.duplicate(), origin, size, fit)
  };

  // Every destination animates the same way - one mechanism for the whole
  // grid, out of whatever frame the window is in.
  animate_to(&target, rect(origin, size), Some(fit));
  Ok(())
}

fn rect(origin: (f64, f64), size: (f64, f64)) -> cg::Rect {
  cg::Rect {
    origin: cg::Point {
      x: origin.0,
      y: origin.1,
    },
    size: cg::Size {
      width: size.0,
      height: size.1,
    },
  }
}

fn session_mut(session: &mut Option<Session>, session_id: u64) -> Result<&mut Session, String> {
  session
    .as_mut()
    .filter(|session| session.id == session_id)
    .ok_or_else(|| "The Glide session has already ended".to_owned())
}
