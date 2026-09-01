// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{sync::Arc, time::Instant};

use cidre::cg;
use core_graphics::geometry::CGPoint;
use tauri::AppHandle;

use super::{
  center::work_area_at,
  cursor::{hide_cursor, is_cursor_pinned, landing_point, pin_cursor, release_cursor},
  native_settings,
  own_window::own_window_at,
  titlebar::{ax_titlebar_at, AxTitlebar},
  tween::{animate_to, WindowTarget},
};
use crate::glide::core::{GlideDetectorOptions, GlideRuntime};
use crate::glide::{begin_logical, finish, finish_with_fade, icon::spawn_icon_lookup};

#[path = "session/access.rs"]
mod access;
#[path = "session/detector.rs"]
mod detector;
#[path = "session/flags.rs"]
mod flags;
#[path = "session/requests.rs"]
mod requests;
#[path = "session/taps.rs"]
mod taps;

pub(super) use access::{accumulate_pointer_travel, active_input, is_active, session_anchor};
pub(super) use detector::{
  set_thirds as set_detector_thirds, settle as settle_detector, update as update_detector,
};
pub(super) use flags::{
  is_suppressing, is_suppressing_momentum, set_momentum_suppression, set_mouse_up_swallow,
  set_suppression, take_mouse_up_swallow,
};
pub(super) use requests::{region_moved, reveal};
pub(super) use taps::register_tap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InputKind {
  Mouse,
  Trackpad,
}

pub(super) struct Session {
  id: u64,
  anchor: CGPoint,
  input: InputKind,
  runtime: GlideRuntime,
  runtime_clock: Instant,
  pointer_travel: f64,
  /// Whether the preview was ever shown. A session that ends before the
  /// detector picks a destination never hid the cursor, so it must not show it.
  revealed: bool,
  /// The window this session drives, captured at the anchor. Hit-testing the
  /// anchor again mid-session would find whatever is under it now, which is no
  /// longer the window the gesture is carrying.
  target: WindowTarget,
  /// Where that window was when the session opened, so a cancel can put it
  /// back exactly.
  original_frame: cg::Rect,
  /// The logical work area of the monitor the anchor lies on, read once. Every
  /// region of the session resolves against this same rectangle.
  work_origin: (f64, f64),
  work_size: (f64, f64),
  /// Whether any region was ever applied. Without one there is nothing for a
  /// cancel to undo, and the window should not be written to at all.
  moved: bool,
}

#[derive(Default)]
pub(super) struct MonitorState {
  session: Option<Session>,
  suppress_momentum: bool,
  /// Whether the trackpad gesture Esc cancelled still has to be swallowed, up
  /// to and including the phase that ends it.
  suppress_gesture: bool,
  /// Whether a mouse glide Esc cancelled still holds Cmd down, so no new
  /// session may begin until the modifier is released.
  suppress_mouse: bool,
  /// The tap a second one can pair with into a double tap, and where it landed.
  tap_candidate: Option<(Instant, CGPoint)>,
  /// Whether the mouse release that ends an intercepted double click still has
  /// to be dropped, so the target application never sees half a click.
  swallow_mouse_up: bool,
  /// Whether a committed preview is still fading out, with the cursor held back
  /// until it is gone. A new session would have to show the very window that is
  /// fading, so it is refused for those few frames instead.
  fading: bool,
}

/// The shared handle the event tap and the reveal command hold the state by.
pub(super) type SharedState = Arc<std::sync::Mutex<MonitorState>>;

static NEXT_SESSION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// The monitor's state, shared with the reveal command. The event tap owns it;
/// this is the handle a command arriving on the main thread reads it through.
static STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();

pub(super) fn shared_state() -> SharedState {
  let state = SharedState::default();
  let _ = STATE.set(state.clone());
  state
}

pub(super) fn begin_if_titlebar(
  app: &AppHandle,
  state: &SharedState,
  input: InputKind,
  anchor: CGPoint,
) -> bool {
  // Turned off, no session ever opens - for either input. The tap keeps
  // running and keeps passing everything through, which is all "off" has to
  // mean.
  if !native_settings::snapshot().enabled {
    return false;
  }
  // The preview the last commit is fading out is the one this session would
  // have to show, and its cursor is still held. Let the fade finish.
  if state.lock().is_ok_and(|state| state.fading) {
    return false;
  }
  // No session, but the cursor is still pinned from the last one: its release
  // has not landed yet, and every event still reports the old anchor as its
  // location. Beginning now would open the session at a point the hand left.
  if is_cursor_pinned() {
    return false;
  }
  // A titlebar point whose window cannot be resolved has nothing to glide, so
  // it does not open a session at all. Screenwide's own windows answer first
  // and without the Accessibility API, which cannot describe them: the pid the
  // preview names is simply ours.
  let Some((target, original_frame, pid)) = target_at(app, anchor) else {
    return false;
  };
  let Some((work_position, work_size)) = work_area_at(app, anchor) else {
    eprintln!("Could not find the monitor the Glide gesture started on");
    return false;
  };
  if let Err(error) = pin_cursor(anchor) {
    eprintln!("Could not pin the Glide cursor: {error}");
    return false;
  }
  let id = NEXT_SESSION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  if let Err(error) = begin_logical(app, id, anchor.x, anchor.y) {
    release_cursor(anchor, false);
    eprintln!("Could not present Glide: {error}");
    return false;
  }
  // The icon lookup is left to run on its own and catch up with the session it
  // names.
  spawn_icon_lookup(app, id, pid);
  if let Ok(mut state) = state.lock() {
    state.session = Some(Session {
      id,
      anchor,
      input,
      runtime: GlideRuntime::new(GlideDetectorOptions {
        rest_ms: native_settings::snapshot().rest_ms,
        ..GlideDetectorOptions::default()
      }),
      runtime_clock: Instant::now(),
      pointer_travel: 0.0,
      revealed: false,
      target,
      original_frame,
      work_origin: (work_position.x, work_position.y),
      work_size: (work_size.width, work_size.height),
      moved: false,
    });
    // A session that got through leaves no cancellation behind it.
    state.suppress_gesture = false;
    state.suppress_mouse = false;
    true
  } else {
    release_cursor(anchor, false);
    finish(app, anchor.x, anchor.y, false);
    false
  }
}

/// What a gesture at this anchor would carry: the window as something the tween
/// can drive, the frame it was read at, and the process whose icon the preview
/// should show. One of Screenwide's own windows is resolved natively and names
/// this very process; everything else comes from the Accessibility hit test,
/// which rejects our own windows outright.
fn target_at(app: &AppHandle, anchor: CGPoint) -> Option<(WindowTarget, cg::Rect, Option<u32>)> {
  match ax_titlebar_at(anchor) {
    AxTitlebar::Titlebar(window, frame) => {
      // The owning process, read before the target takes the element over, so
      // the preview can be told which application it is carrying.
      let pid = window.pid().ok().and_then(|pid| u32::try_from(pid).ok());
      Some((WindowTarget::new(window), frame, pid))
    }
    // Our own pid under the cursor: only now are the native resolver's
    // main-thread hops paid, and never on samples over other applications.
    AxTitlebar::OwnProcess => own_window_at(app, anchor)
      .map(|(window, frame)| (WindowTarget::own(window), frame, Some(std::process::id()))),
    AxTitlebar::Miss => None,
  }
}

/// Ends the live session, if there is one. `cancelled` rides out with the end
/// so the detector knows whether to commit what it had armed.
pub(super) fn end_session(app: &AppHandle, state: &SharedState, cancelled: bool) {
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
    // The restore puts back a frame the window already held, so it needs no
    // gravity correction, and the preview it would report to is going away.
    animate_to(&session.target, session.original_frame, None);
  }
  // A committed glide that had something on screen fades out, and the cursor
  // waits for it so it never appears over a half-faded preview. Cancelled ends
  // and sessions that were never revealed dismiss instantly, as before.
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
  // Read here rather than inside the closure: the setting that counts is the
  // one in force when the gesture ended.
  let cursor_follows = native_settings::snapshot().cursor_follows;
  finish_with_fade(
    app,
    anchor.x,
    anchor.y,
    // The cursor comes back a beat into the fade - and a commit that carried
    // the window lands it there, holding the same grip, ready to keep using
    // what it just placed. A frame that cannot be read falls back to the
    // anchor, as does a commit that moved nothing or one the user asked to
    // leave the cursor behind.
    Box::new(move || {
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
