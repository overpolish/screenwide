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
  core::{corrected_origin, frame_fits, frame_fractions, GlideFrame},
  fit::{emit_fit, FitRect, GlideFitEvent},
  region_rect::RegionGravity,
};

#[path = "tween/destination.rs"]
mod destination;

/// How long a move takes, end to end. Short enough to feel like a response to
/// the gesture rather than a transition being watched.
const TWEEN_SECONDS: f64 = 0.18;
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
  landing: Option<POINT>,
}

/// The one tween in flight, if any. Never held across a `SetWindowPos`: that
/// call blocks on the target application's thread.
static TWEEN: Mutex<Option<Tween>> = Mutex::new(None);
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
      landing: None,
    });
  }
}

/// Where the window in flight is headed, so a commit that lands while it is
/// still travelling can aim the cursor at the frame it will end up with.
pub(super) fn in_flight_destination() -> Option<GlideFrame> {
  TWEEN
    .lock()
    .ok()
    .and_then(|slot| slot.as_ref().map(|tween| tween.destination))
}

/// Puts the cursor at its landing point: now, or once the window in flight
/// has arrived there.
pub(super) fn land_cursor(point: POINT) {
  let deferred = TWEEN.lock().ok().is_some_and(|mut slot| {
    slot
      .as_mut()
      .map(|tween| tween.landing = Some(point))
      .is_some()
  });
  if !deferred {
    let _ = unsafe { SetCursorPos(point.x, point.y) };
  }
}

/// Drops whatever is in flight without moving the window any further. A
/// minimize that follows must not be fought by a late frame.
pub(super) fn cancel() {
  GENERATION.fetch_add(1, Ordering::Relaxed);
  if let Ok(mut slot) = TWEEN.lock() {
    *slot = None;
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
    interpolate(tween.start, tween.destination, eased(progress))
  };

  let placed = if tween.resizes {
    tween.target.set_frame(frame)
  } else {
    tween.target.set_origin(frame)
  };
  if !is_current(tween.generation) || placed.is_err() {
    return;
  }
  if settling {
    if let Some(context) = tween.fit.take() {
      settle(&tween.target, tween.requested, tween.generation, &context);
    }
    if let Some(point) = tween.landing {
      let _ = unsafe { SetCursorPos(point.x, point.y) };
    }
    return;
  }
  if let Ok(mut slot) = TWEEN.lock() {
    if slot.is_none() {
      *slot = Some(tween);
    }
  }
}

/// Reads the settled frame, corrects its origin if the window came out a
/// different size than asked for, and reports the result to the preview. The
/// generation is re-checked around every window call: a retarget that landed
/// while the frame was being read owns the window now.
fn settle(target: &WindowTarget, requested: GlideFrame, generation: u64, context: &FitContext) {
  let Ok(achieved) = target.frame() else {
    return;
  };
  if !is_current(generation) {
    return;
  }
  let fits = frame_fits(achieved, requested, FIT_EPSILON);
  let mut frame = achieved;
  if !fits {
    let (x, y) = corrected_origin(requested, achieved, context.gravity);
    frame.x = x;
    frame.y = y;
    if target.set_origin(frame).is_err() || !is_current(generation) {
      return;
    }
  }
  let Some(actual) = frame_fractions(
    frame,
    (context.work.x, context.work.y),
    (context.work.width, context.work.height),
  ) else {
    return;
  };
  if let Err(error) = emit_fit(
    &context.app,
    GlideFitEvent {
      session_id: context.session_id,
      fits,
      actual: FitRect {
        x: actual.x,
        y: actual.y,
        width: actual.width,
        height: actual.height,
      },
    },
  ) {
    eprintln!("Could not report the Glide placement: {error}");
  }
}

fn is_current(generation: u64) -> bool {
  GENERATION.load(Ordering::Relaxed) == generation
}

fn interpolate(start: GlideFrame, destination: GlideFrame, fraction: f64) -> GlideFrame {
  GlideFrame {
    x: lerp(start.x, destination.x, fraction),
    y: lerp(start.y, destination.y, fraction),
    width: lerp(start.width, destination.width, fraction),
    height: lerp(start.height, destination.height, fraction),
  }
}

fn lerp(from: f64, to: f64, fraction: f64) -> f64 {
  from + (to - from) * fraction
}

/// Ease-out cubic: the window leaves the old region fast and arrives softly.
fn eased(fraction: f64) -> f64 {
  let remaining = 1.0 - fraction.clamp(0.0, 1.0);
  1.0 - remaining * remaining * remaining
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_ease_starts_and_ends_on_its_endpoints() {
    assert_eq!(eased(0.0), 0.0);
    assert_eq!(eased(1.0), 1.0);
  }

  #[test]
  fn interpolation_lands_on_the_destination() {
    let start = GlideFrame {
      x: 0.0,
      y: 0.0,
      width: 100.0,
      height: 100.0,
    };
    let destination = GlideFrame {
      x: 40.0,
      y: 20.0,
      width: 300.0,
      height: 200.0,
    };
    assert_eq!(interpolate(start, destination, 1.0), destination);
    assert_eq!(interpolate(start, destination, 0.0), start);
  }
}
