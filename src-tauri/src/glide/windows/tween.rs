// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The animation every Windows Glide move runs through. `SetWindowPos` only
//! teleports a window, so the motion is ours: one app-lifetime thread steps a
//! single in-flight tween towards its destination, and a new destination
//! arriving mid-flight preempts it from wherever the window is now. This is
//! the Windows twin of the macOS tween, working in physical pixels.

use std::{
  sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
  },
  time::{Duration, Instant},
};

use tauri::AppHandle;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::SetCursorPos};

use super::target::WindowTarget;
use crate::glide::{
  core::activity::BusyLease,
  core::{corrected_origin, frame_fits, frame_fractions, GlideFrame},
  fit::{emit_fit, FitRect, GlideFitEvent},
  region_rect::RegionGravity,
};

#[path = "tween/completion.rs"]
mod completion;
#[path = "tween/destination.rs"]
mod destination;
#[path = "tween/settle.rs"]
mod settle;
use settle::settle;

pub(super) use completion::after_current;

/// How long a move takes, end to end. Short enough to feel like a response to
/// the gesture rather than a transition being watched.
const TWEEN_SECONDS: f64 = 0.22;
/// One step per display frame, near enough.
const STEP_INTERVAL: Duration = Duration::from_millis(16);
/// How far the achieved size may miss the destination and still count as a
/// fill. Applications round their size to a character cell or a device pixel,
/// and a pixel or two of that is not a constraint worth showing.
const FIT_EPSILON: f64 = 2.0;

/// Everything the settling step needs to place a constrained window and report
/// it. A move with no region behind it - an Esc restore - carries none of this
/// and settles exactly where it was sent.
pub(super) struct FitContext {
  pub app: AppHandle,
  pub session_id: u64,
  pub gravity: RegionGravity,
  pub work: GlideFrame,
}

struct Tween {
  target: WindowTarget,
  /// Where the window was when this tween took over, read once so the curve
  /// starts from the real frame rather than the last one we asked for.
  start: GlideFrame,
  destination: GlideFrame,
  requested: GlideFrame,
  resizes: bool,
  started_at: Instant,
  generation: u64,
  fit: Option<FitContext>,
  /// Where the cursor goes once the window arrives. A commit that lands while
  /// the window is still travelling leaves the cursor for the settle to place,
  /// so it never sits on a frame the window has not reached.
  _busy: BusyLease,
  _completion: completion::Ticket,
}

struct Active {
  generation: u64,
  destination: GlideFrame,
  landing: Option<(POINT, BusyLease)>,
}

/// The one tween in flight, if any. Never held across a `SetWindowPos`: that
/// call blocks on the target application's thread.
static TWEEN: Mutex<Option<Tween>> = Mutex::new(None);
/// Metadata remains present while `step` owns the tween outside the mutex.
/// Hook callbacks therefore defer cursor landing instead of warping early.
static ACTIVE: Mutex<Option<Active>> = Mutex::new(None);
/// Bumped by every retarget, so a step that was already applying a frame when
/// the destination changed drops its result instead of fighting the new tween.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub(super) fn start() {
  if let Err(error) = std::thread::Builder::new()
    .name("glide-tween".to_owned())
    .spawn(run)
  {
    eprintln!("Could not start Glide window animation: {error}");
  }
}

fn run() {
  loop {
    std::thread::sleep(STEP_INTERVAL);
    step();
  }
}

/// Animates a window to a rectangle, preempting whatever was in flight. The
/// current frame is read here rather than carried over from the old tween, so
/// a retarget curves out of where the window actually is.
pub(super) fn animate_to(target: WindowTarget, destination: GlideFrame, fit: Option<FitContext>) {
  let start = match target.prepare_for_move() {
    Ok(start) => start,
    Err(error) => {
      eprintln!("Could not move the Glide window: {error}");
      return;
    }
  };
  let (travel, resizes) = fit.as_ref().map_or((destination, true), |context| {
    destination::travel(start, destination, target.is_resizable(), context.gravity)
  });
  let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
  if let Ok(mut slot) = TWEEN.lock() {
    *slot = Some(Tween {
      target,
      start,
      destination: travel,
      requested: destination,
      resizes,
      started_at: Instant::now(),
      generation,
      fit,
      _busy: BusyLease::acquire(),
      _completion: completion::begin(),
    });
    if let Ok(mut active) = ACTIVE.lock() {
      *active = Some(Active {
        generation,
        destination: travel,
        landing: None,
      });
    }
  }
}

pub(super) fn in_flight_destination() -> Option<GlideFrame> {
  ACTIVE
    .lock()
    .ok()
    .and_then(|slot| slot.as_ref().map(|active| active.destination))
}

pub(super) fn land_cursor(point: POINT, busy: BusyLease) {
  let mut busy = Some(busy);
  let deferred = ACTIVE.lock().ok().is_some_and(|mut slot| {
    slot.as_mut().is_some_and(|active| {
      active.landing = busy.take().map(|lease| (point, lease));
      active.landing.is_some()
    })
  });
  if !deferred {
    crate::recording::cursor::set_cursor_visibility(false, None);
    let _ = unsafe { SetCursorPos(point.x, point.y) };
    crate::recording::cursor::set_cursor_visibility(
      true,
      Some((f64::from(point.x), f64::from(point.y))),
    );
    drop(busy.take());
  }
}

/// Drops whatever is in flight without moving the window any further. A
/// minimize that follows must not be fought by a late frame.
pub(super) fn cancel() {
  GENERATION.fetch_add(1, Ordering::Relaxed);
  if let Ok(mut slot) = TWEEN.lock() {
    *slot = None;
  }
  if let Ok(mut active) = ACTIVE.lock() {
    *active = None;
  }
}

/// One frame of the animation. The tween is taken out of the lock for the
/// duration of the window calls and only put back if nothing claimed the slot
/// meanwhile, which is exactly how a retarget wins.
fn step() {
  let Some(mut tween) = TWEEN.lock().ok().and_then(|mut slot| slot.take()) else {
    return;
  };
  let progress = tween.started_at.elapsed().as_secs_f64() / TWEEN_SECONDS;
  let settling = progress >= 1.0;
  let frame = if settling {
    tween.destination
  } else {
    destination::interpolate(tween.start, tween.destination, destination::eased(progress))
  };

  let placed = if tween.resizes {
    tween.target.set_frame(frame)
  } else {
    tween.target.set_origin(frame)
  };
  if !is_current(tween.generation) || placed.is_err() {
    clear_active(tween.generation);
    return;
  }
  if settling {
    if let Some(context) = tween.fit.take() {
      settle(&tween.target, tween.requested, tween.generation, &context);
    }
    let landing = ACTIVE
      .lock()
      .ok()
      .and_then(|mut active| {
        if active
          .as_ref()
          .is_some_and(|active| active.generation == tween.generation)
        {
          active.take()
        } else {
          None
        }
      })
      .and_then(|active| active.landing);
    if let Some((point, busy)) = landing {
      crate::recording::cursor::set_cursor_visibility(false, None);
      let _ = unsafe { SetCursorPos(point.x, point.y) };
      crate::recording::cursor::set_cursor_visibility(
        true,
        Some((f64::from(point.x), f64::from(point.y))),
      );
      drop(busy);
    }
    return;
  }
  if let Ok(mut slot) = TWEEN.lock() {
    if slot.is_none() && is_current(tween.generation) {
      *slot = Some(tween);
    }
  }
}

fn is_current(generation: u64) -> bool {
  GENERATION.load(Ordering::Relaxed) == generation
}

fn clear_active(generation: u64) {
  if let Ok(mut active) = ACTIVE.lock() {
    if active
      .as_ref()
      .is_some_and(|active| active.generation == generation)
    {
      *active = None;
    }
  }
}
