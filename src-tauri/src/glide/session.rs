// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A Glide session as every platform runs it: what opens one, how the
//! detector's effects reach the preview and the window, and how it ends. A
//! platform supplies its window, its pointer and its monitor picker through
//! [`GlidePlatform`], and keeps its input hooks to itself.

mod center;
mod end;
mod running;

pub(crate) use center::center;
pub(crate) use end::end;
use running::finish_opening;
pub(crate) use running::{set_thirds, settle, update};

use std::time::Instant;

use tauri::AppHandle;

use crate::glide::core::tween::GlideWindow;
use crate::glide::core::{GlideFrame, GlideRuntime};
use crate::glide::region_rect::{PlacedRegion, RegionGravity};
use crate::glide::{finish, icon::spawn_icon_lookup, present_locked};

/// A point in the platform's own units: logical points on macOS, physical
/// pixels on Windows.
pub(crate) type Point = (f64, f64);

pub(crate) trait GlidePlatform: Sized + 'static {
  type Window: GlideWindow;
  type Input: Copy + PartialEq + std::fmt::Debug + Send + 'static;
  type Monitors: Send + 'static;
  /// What a platform keeps on a session beyond the shared fields.
  type Extra: Default + Send + 'static;
  /// The platform's Glide state, the session slot among it.
  type State: Send + 'static;

  /// Runs `f` under the platform's state lock.
  fn with_state<R>(f: impl FnOnce(&mut Self::State) -> R) -> Option<R>;
  fn slot(state: &mut Self::State) -> &mut Option<Session<Self>>;

  /// Whether a platform rule keeps a session from opening now: the setting,
  /// a capture in progress, another gesture still finishing.
  fn blocks_begin(app: &AppHandle) -> bool;
  /// The window under `anchor`, the frame it was read at and the process
  /// whose icon the preview shows.
  fn target_at(app: &AppHandle, anchor: Point) -> Option<(Self::Window, GlideFrame, Option<u32>)>;
  /// Whether the session picks a monitor rather than a region: `Some(true)`
  /// to capture the monitors, `None` when the gesture must not open at all.
  fn monitor_request() -> Option<bool>;
  fn capture_monitors(app: &AppHandle, anchor: Point) -> Option<Self::Monitors>;
  /// The work area of the monitor the session opens on.
  fn work_area(app: &AppHandle, anchor: Point, target: &Self::Window) -> Option<GlideFrame>;
  /// Holds the pointer at `anchor` for the session, where the platform does.
  fn hold_pointer(anchor: Point) -> Result<(), String>;
  /// Places the preview at `anchor` and tells the frontend the session began.
  fn present(app: &AppHandle, id: u64, anchor: Point) -> Result<(), String>;
  /// A session opened: whatever an earlier cancellation left to swallow is
  /// done with.
  fn session_began(state: &mut Self::State);

  /// The detector reports a destination worth a pulse.
  fn on_ready(app: &AppHandle);
  /// A detector sample arrived; `moved` when it travelled.
  fn on_sample(extra: &mut Self::Extra, moved: bool);
  /// Hands the detector's monitor effects to the picker. Answers whether
  /// the picker took them, in which case regions are not placed.
  fn monitor_effects(app: &AppHandle, id: u64, step: Option<(i8, i8)>, ready: bool) -> bool;
  /// Hides the pointer for the revealed preview. Runs under the state lock,
  /// so an end on another thread cannot slip between the mark and the hide.
  fn hide_pointer();
  fn show_preview(app: &AppHandle, id: u64, input: Self::Input) -> Result<(), String>;
  /// Where a region places the session's window, and the edge it is pulled
  /// to when the window cannot fill it.
  fn region_frame(session: &Session<Self>, region: &PlacedRegion) -> (GlideFrame, RegionGravity);

  /// The session left its slot. Runs under the state lock.
  fn session_taken(state: &mut Self::State, session: &Session<Self>, cancelled: bool);
  /// Ends the monitor picker. A commit carries the window to the picked
  /// monitor. Answers whether the picker was showing.
  fn end_monitors(app: &AppHandle, session: &mut Session<Self>, cancelled: bool) -> bool;
  /// Lets go of the pointer, leaving it at `anchor` where the platform held
  /// it there, and shows it when `show`.
  fn release_pointer(anchor: Point, show: bool);
  /// Puts the pointer at `point` and shows it.
  fn land_pointer(point: Point);
  /// Moves the pointer onto a window that moved out from under it, leaving
  /// its visibility alone.
  fn carry_pointer(point: Point);
  /// The preview faded out. `completion` holds new sessions off until the
  /// platform has finished tearing it down.
  fn after_fade(app: &AppHandle, id: u64, completion: crate::glide::BusyLease);
}

pub(crate) struct Session<P: GlidePlatform> {
  pub(crate) id: u64,
  pub(crate) anchor: Point,
  pub(crate) input: P::Input,
  pub(crate) runtime: GlideRuntime,
  pub(crate) runtime_clock: Instant,
  pub(crate) pointer_travel: f64,
  /// Whether the preview was ever shown. A session that ends before the
  /// detector picks a destination never hid the pointer, so it must not
  /// show it.
  pub(crate) revealed: bool,
  /// The window this session drives, captured at the anchor. Hit-testing the
  /// anchor again mid-session would find whatever is under it now.
  pub(crate) target: P::Window,
  /// Where that window was when the session opened, so a cancel can put it
  /// back exactly.
  pub(crate) original_frame: GlideFrame,
  /// The work area of the monitor the anchor lies on, read once. Every
  /// region of the session resolves against it.
  pub(crate) work: GlideFrame,
  /// Whether any region was ever applied. Without one there is nothing for a
  /// cancel to undo, and the window is not written to at all.
  pub(crate) moved: bool,
  /// Whether the last region placed the window back where it started.
  pub(crate) returned_to_origin: bool,
  pub(crate) monitors: Option<P::Monitors>,
  pub(crate) extra: P::Extra,
}

impl<P: GlidePlatform> Session<P> {
  fn timestamp(&self) -> f64 {
    self.runtime_clock.elapsed().as_secs_f64() * 1_000.0
  }
}

static NEXT_SESSION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn next_id() -> u64 {
  NEXT_SESSION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Runs `f` on the live session, if there is one.
pub(crate) fn with_session<P: GlidePlatform, R>(f: impl FnOnce(&mut Session<P>) -> R) -> Option<R> {
  P::with_state(|state| P::slot(state).as_mut().map(f)).flatten()
}

pub(crate) fn with_session_id<P: GlidePlatform, R>(
  id: u64,
  f: impl FnOnce(&mut Session<P>) -> R,
) -> Result<R, String> {
  P::with_state(|state| {
    P::slot(state)
      .as_mut()
      .filter(|session| session.id == id)
      .map(f)
  })
  .ok_or_else(|| "The Glide session state is unavailable".to_owned())?
  .ok_or_else(|| "The Glide session has already ended".to_owned())
}

/// Opens a session for the window under `anchor`, if anything may open one.
pub(crate) fn begin<P: GlidePlatform>(app: &AppHandle, input: P::Input, anchor: Point) -> bool {
  if crate::capture_overlays::blocks_glide(app)
    || crate::glide::BusyLease::is_busy()
    || P::blocks_begin(app)
    || with_session::<P, _>(|_| ()).is_some()
  {
    return false;
  }
  // A point whose window cannot be resolved has nothing to glide, so it opens
  // no session at all.
  let Some((target, original_frame, pid)) = P::target_at(app, anchor) else {
    return false;
  };
  // A window its application will not let us place has nothing to carry, so
  // no session opens for it - the pointer is never held and nothing is
  // raised. The gesture still gets an answer: the preview shows the lock on
  // the anchor and fades, holding the activity lease meanwhile.
  if !target.is_movable() {
    eprintln!("The application does not allow moving this window");
    present_locked(app, next_id(), anchor.0, anchor.1);
    return false;
  }
  let Some(wants_monitors) = P::monitor_request() else {
    return false;
  };
  let monitors = if wants_monitors {
    let Some(selection) = P::capture_monitors(app, anchor) else {
      return false;
    };
    Some(selection)
  } else {
    None
  };
  target.raise();
  let Some(work) = P::work_area(app, anchor, &target) else {
    eprintln!("Could not find the monitor the Glide gesture started on");
    return false;
  };
  if let Err(error) = P::hold_pointer(anchor) {
    eprintln!("Could not hold the Glide pointer: {error}");
    return false;
  }
  let id = next_id();
  if let Err(error) = P::present(app, id, anchor) {
    P::release_pointer(anchor, false);
    eprintln!("Could not present Glide: {error}");
    return false;
  }
  let mut runtime = GlideRuntime::default();
  if monitors.is_some() {
    runtime.begin_monitor_navigation();
  }
  let session = Session::<P> {
    id,
    anchor,
    input,
    runtime,
    runtime_clock: Instant::now(),
    pointer_travel: 0.0,
    revealed: false,
    target,
    original_frame,
    work,
    moved: false,
    returned_to_origin: false,
    monitors,
    extra: P::Extra::default(),
  };
  let opened = P::with_state(|state| {
    let slot = P::slot(state);
    if slot.is_some() {
      return false;
    }
    *slot = Some(session);
    P::session_began(state);
    true
  })
  .unwrap_or(false);
  if opened {
    spawn_icon_lookup(app, id, pid);
  } else {
    P::release_pointer(anchor, false);
    finish(app, anchor.0, anchor.1, true);
  }
  opened
}
