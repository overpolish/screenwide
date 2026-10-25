// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS's half of a Glide session: the window under the Accessibility hit
//! test, the pinned cursor, haptics and the monitor picker, under the shared
//! session flow in `glide::session`. Coordinates are logical points.

use std::{sync::Arc, time::Instant};

use core_graphics::geometry::CGPoint;
use tauri::AppHandle;

use super::{
  center::work_area_at,
  cursor::{carry_cursor, hide_cursor, is_cursor_pinned, pin_cursor, release_cursor},
  native_settings,
  tween::{glide_frame, WindowTarget},
};
use crate::glide::core::GlideFrame;
use crate::glide::region_rect::{region_gravity, region_rect, PlacedRegion, RegionGravity};
use crate::glide::session::{self as shared, GlidePlatform, Point};

mod access;
mod flags;
pub(super) mod monitor_input;
pub(super) mod monitors;
pub(super) mod mouse_preview;
mod taps;
mod target;

pub(super) use target::target_at;

pub(super) use access::{
  accumulate_pointer_travel, active_input, is_active, monitor_mode, mouse_center_context,
  session_anchor,
};
pub(super) use flags::{
  is_suppressing, is_suppressing_momentum, set_momentum_suppression, set_mouse_up_swallow,
  set_suppression, take_mouse_up_swallow,
};
pub(super) use taps::register_tap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputKind {
  Mouse,
  Trackpad,
  Wheel,
}

pub(super) struct MacGlide;

pub(super) type Session = shared::Session<MacGlide>;

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
  /// The committed session still presenting while its move settles and fades.
  /// Its queued reveal remains valid, and new sessions wait for teardown.
  fading: Option<u64>,
}

/// The shared handle the event tap and the reveal command hold the state by.
pub(super) type SharedState = Arc<std::sync::Mutex<MonitorState>>;

/// The monitor's state, shared with the reveal command. The event tap owns it;
/// this is the handle a command arriving on the main thread reads it through.
static STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();

pub(super) fn shared_state() -> SharedState {
  let state = SharedState::default();
  let _ = STATE.set(state.clone());
  state
}

fn cg_point((x, y): Point) -> CGPoint {
  CGPoint::new(x, y)
}

impl GlidePlatform for MacGlide {
  type Window = WindowTarget;
  type Input = InputKind;
  type Monitors = monitors::Selection;
  type Extra = ();
  type State = MonitorState;

  fn with_state<R>(f: impl FnOnce(&mut MonitorState) -> R) -> Option<R> {
    STATE.get()?.lock().ok().map(|mut state| f(&mut state))
  }

  fn slot(state: &mut MonitorState) -> &mut Option<Session> {
    &mut state.session
  }

  fn blocks_begin(_app: &AppHandle) -> bool {
    // Turned off, no session ever opens - for either input. The tap keeps
    // running and keeps passing everything through, which is all "off" has
    // to mean.
    !native_settings::snapshot().enabled
      || super::deliberate::blocks_sessions()
      // The preview the last commit is fading out is the one this session
      // would have to show, and its cursor is still held.
      || Self::with_state(|state| state.fading.is_some()).unwrap_or(false)
      // The cursor is still pinned from the last session: its release has
      // not landed, and every event still reports the old anchor.
      || is_cursor_pinned()
      || crate::glide::picker_cards::is_dismissing()
  }

  fn target_at(app: &AppHandle, anchor: Point) -> Option<(WindowTarget, GlideFrame, Option<u32>)> {
    target_at(app, cg_point(anchor)).map(|(target, frame, pid)| (target, glide_frame(frame), pid))
  }

  fn monitor_request() -> Option<bool> {
    Some(native_settings::is_down(
      native_settings::snapshot().monitors_modifier,
    ))
  }

  fn capture_monitors(app: &AppHandle, anchor: Point) -> Option<monitors::Selection> {
    monitors::capture(app, cg_point(anchor))
  }

  fn work_area(app: &AppHandle, anchor: Point, _target: &WindowTarget) -> Option<GlideFrame> {
    work_area_at(app, cg_point(anchor)).map(|(position, size)| GlideFrame {
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
    })
  }

  fn hold_pointer(anchor: Point) -> Result<(), String> {
    pin_cursor(cg_point(anchor))
  }

  fn present(app: &AppHandle, id: u64, anchor: Point) -> Result<(), String> {
    crate::glide::begin_logical(app, id, anchor.0, anchor.1)
  }

  fn session_began(state: &mut MonitorState) {
    // A session that got through leaves no cancellation behind it.
    state.suppress_gesture = false;
    state.suppress_mouse = false;
  }

  fn on_ready(app: &AppHandle) {
    if native_settings::snapshot().haptics {
      let _ = super::haptic(app);
    }
  }

  fn on_sample(_extra: &mut (), _moved: bool) {}

  fn monitor_effects(app: &AppHandle, id: u64, step: Option<(i8, i8)>, ready: bool) -> bool {
    monitors::update(app, id, step, ready)
  }

  fn hide_pointer() {
    if let Err(error) = hide_cursor() {
      eprintln!("{error}");
    }
  }

  fn show_preview(app: &AppHandle, id: u64, input: InputKind) -> Result<(), String> {
    let main_app = app.clone();
    app
      .run_on_main_thread(move || {
        // A committed session still owns its queued reveal while settling and
        // fading. Cancellation must continue to invalidate a late show.
        let active = Self::with_state(|state| {
          state
            .session
            .as_ref()
            .is_some_and(|session| session.id == id)
            || state.fading == Some(id)
        })
        .unwrap_or(false);
        if active {
          if let Err(error) =
            crate::app_windows::show_glide_island(&main_app, input == InputKind::Mouse)
          {
            eprintln!("Could not present Glide: {error}");
          }
        }
      })
      .map_err(|error| error.to_string())
  }

  fn region_frame(session: &Session, region: &PlacedRegion) -> (GlideFrame, RegionGravity) {
    let work = session.work;
    let (origin, size) = region_rect(
      (work.x, work.y),
      (work.width, work.height),
      region,
      // The gap is read per placement, so a change lands on the next
      // transition.
      native_settings::snapshot().window_gap,
    );
    (
      GlideFrame {
        x: origin.0,
        y: origin.1,
        width: size.0,
        height: size.1,
      },
      region_gravity(region),
    )
  }

  fn session_taken(state: &mut MonitorState, session: &Session, cancelled: bool) {
    if !cancelled && session.revealed {
      state.fading = Some(session.id);
    }
    // Only a revealed glide breaks a double tap in the making. A tap sheds a
    // few scroll events of its own, and the invisible session they open must
    // not clear the candidate that same tap just stored.
    if session.revealed {
      state.tap_candidate = None;
    }
  }

  fn end_monitors(app: &AppHandle, session: &mut Session, cancelled: bool) -> bool {
    let showing = session
      .monitors
      .as_ref()
      .is_some_and(|selection| selection.active);
    if showing {
      crate::glide::picker_cards::dismiss(app, cancelled);
      if !cancelled {
        monitors::commit(app, session);
      }
    }
    showing
  }

  fn release_pointer(anchor: Point, show: bool) {
    release_cursor(cg_point(anchor), show);
  }

  fn land_pointer(point: Point) {
    release_cursor(cg_point(point), true);
  }

  fn carry_pointer(point: Point) {
    carry_cursor(cg_point(point));
  }

  fn after_fade(_app: &AppHandle, _id: u64, completion: crate::glide::BusyLease) {
    crate::glide::picker_cards::after_dismissed(Box::new(move || {
      let _completion = completion;
      Self::with_state(|state| state.fading = None);
    }));
  }
}

pub(super) fn begin_if_titlebar(app: &AppHandle, input: InputKind, anchor: CGPoint) -> bool {
  shared::begin::<MacGlide>(app, input, (anchor.x, anchor.y))
}

pub(super) fn update_detector(app: &AppHandle, delta_x: f64, delta_y: f64, thirds: bool) {
  shared::update::<MacGlide>(app, delta_x, delta_y, thirds);
}

pub(super) fn set_detector_thirds(app: &AppHandle, thirds: bool) {
  shared::set_thirds::<MacGlide>(app, thirds);
}

/// Resting fingers produce no input callback, so the event tap's 16-ms
/// run-loop poll completes the detector's rest gate.
pub(super) fn settle_detector(app: &AppHandle) {
  shared::settle::<MacGlide>(app);
}

/// Ends the live session, if there is one. `cancelled` rides out with the end
/// so the detector knows whether to commit what it had armed.
pub(super) fn end_session(app: &AppHandle, cancelled: bool) {
  shared::end::<MacGlide>(app, cancelled, native_settings::snapshot().cursor_follows);
}

pub(super) fn cancel_current(app: &AppHandle) {
  end_session(app, true);
}
