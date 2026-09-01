// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The animation every Glide window move runs through. The Accessibility API
//! only teleports a window, so the motion is ours: one app-lifetime thread
//! steps a single in-flight tween towards its destination, and a new
//! destination arriving mid-flight preempts it from wherever the window is now.

use std::{
  sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
  },
  time::{Duration, Instant},
};

use cidre::cg;

#[path = "tween/fit.rs"]
mod fit;
#[path = "tween/target.rs"]
mod target;

pub(super) use fit::FitContext;
pub(super) use target::WindowTarget;

/// How long a move takes, end to end. Short enough to feel like a response to
/// the gesture rather than a transition being watched.
const TWEEN_SECONDS: f64 = 0.18;
/// One step per display frame, near enough.
const STEP_INTERVAL: Duration = Duration::from_millis(16);

struct Tween {
  target: WindowTarget,
  /// Where the window was when this tween took over, read once so the curve
  /// starts from the real frame rather than the last one we asked for.
  start: cg::Rect,
  /// Where the window is being carried. For a window its application refuses
  /// to resize this is the region rectangle reduced to a move: the size the
  /// window already has, placed at the region's gravity.
  destination: cg::Rect,
  /// The rectangle the placement asked for, which is what the settling step
  /// judges the achieved frame against - a window that could not take the
  /// region's size must still be reported as not fitting it.
  requested: cg::Rect,
  started_at: Instant,
  /// Whether the window changes size on the way. A pure move leaves the size
  /// attribute alone, so a window that refuses to be resized still travels.
  resizes: bool,
  generation: u64,
  /// What the settling step needs to place a window its application refused to
  /// resize, and to report the frame it ended up with. A move that belongs to
  /// no region carries none.
  fit: Option<FitContext>,
}

/// The one tween in flight, if any. Never held across an Accessibility call:
/// the event tap takes this lock too, and a blocked tap freezes the pointer.
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
/// a retarget curves out of where the window actually is. `fit` is the region
/// context the settling step corrects and reports against; a move that is not a
/// region placement passes `None` and settles as it always did. A window its
/// application refuses to resize keeps its size and travels anyway, as long as
/// there is a region to align it inside.
pub(super) fn animate_to(target: &WindowTarget, destination: cg::Rect, fit: Option<FitContext>) {
  if !target.is_movable() {
    eprintln!("The application does not allow moving this window");
    return;
  }
  let Some(start) = target.frame() else {
    eprintln!("Could not read the window to move");
    return;
  };
  let mut resizes = start.size != destination.size;
  let mut travel = destination;
  if resizes && !target.is_resizable() {
    // A window whose application fixes its size still belongs in the region it
    // was thrown at: it travels there at the size it has, aligned to the
    // region's gravity the same way an undersized one is corrected on settle.
    let Some(context) = fit.as_ref() else {
      eprintln!("The application does not allow resizing this window");
      return;
    };
    travel = cg::Rect {
      origin: fit::corrected_origin(destination, start.size, context.gravity),
      size: start.size,
    };
    resizes = false;
  }

  let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
  if let Ok(mut slot) = TWEEN.lock() {
    *slot = Some(Tween {
      target: target.duplicate(),
      start,
      destination: travel,
      requested: destination,
      started_at: Instant::now(),
      resizes,
      generation,
      fit,
    });
  }
}

/// One frame of the animation. The tween is taken out of the lock for the
/// duration of the Accessibility calls and only put back if nothing claimed the
/// slot meanwhile, which is exactly how a retarget wins.
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

  if !is_current(tween.generation) || !apply(&mut tween, frame, settling) {
    return;
  }
  if settling {
    // Still outside the lock, and still guarded: reading the window back and
    // correcting it is more Accessibility traffic, which a retarget may
    // overtake at any point.
    if let Some(context) = tween.fit.take() {
      fit::settle(
        &mut tween.target,
        tween.requested,
        tween.generation,
        &context,
      );
    }
    return;
  }
  if let Ok(mut slot) = TWEEN.lock() {
    if slot.is_none() {
      *slot = Some(tween);
    }
  }
}

/// Whether a tween is still the one in flight. Every write a step makes is
/// preceded by this: a retarget that landed meanwhile owns the window now.
fn is_current(generation: u64) -> bool {
  GENERATION.load(Ordering::Relaxed) == generation
}

/// Writes one frame onto the window. The settling step repeats the position
/// after the resize: an application clamps a move against the size it still
/// has, so a window travelling to a smaller region stops short of the
/// destination, and the resize that follows can shift the origin again as the
/// window is pulled back on screen. Repeating the position once the window is
/// the right size settles it exactly where it belongs.
fn apply(tween: &mut Tween, frame: cg::Rect, settling: bool) -> bool {
  if !tween.target.set_pos(&frame.origin) {
    return false;
  }
  if !tween.resizes {
    return true;
  }
  if !tween.target.set_size(&frame.size) {
    return false;
  }
  !settling || tween.target.set_pos(&frame.origin)
}

fn interpolate(start: cg::Rect, destination: cg::Rect, fraction: f64) -> cg::Rect {
  cg::Rect {
    origin: cg::Point {
      x: lerp(start.origin.x, destination.origin.x, fraction),
      y: lerp(start.origin.y, destination.origin.y, fraction),
    },
    size: cg::Size {
      width: lerp(start.size.width, destination.size.width, fraction),
      height: lerp(start.size.height, destination.size.height, fraction),
    },
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
  fn the_ease_is_ahead_of_a_linear_one_throughout() {
    for step in 1..10 {
      let fraction = f64::from(step) / 10.0;
      assert!(eased(fraction) > fraction);
    }
  }

  #[test]
  fn interpolation_lands_on_the_destination() {
    let start = cg::Rect {
      origin: cg::Point { x: 0.0, y: 0.0 },
      size: cg::Size {
        width: 100.0,
        height: 100.0,
      },
    };
    let destination = cg::Rect {
      origin: cg::Point { x: 40.0, y: 20.0 },
      size: cg::Size {
        width: 300.0,
        height: 200.0,
      },
    };
    assert_eq!(interpolate(start, destination, 1.0), destination);
    assert_eq!(interpolate(start, destination, 0.0), start);
  }
}
