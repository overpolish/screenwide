// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The animation every Glide window move runs through. Neither platform's
//! window API animates - the Accessibility API and `SetWindowPos` only
//! teleport a window - so the motion is ours: one app-lifetime thread steps a
//! single in-flight tween towards its destination, and a new destination
//! arriving mid-flight preempts it from wherever the window is now. A platform
//! supplies only the window, through [`GlideWindow`].

use std::{
  sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
  },
  time::Instant,
};

use super::tween_completion as completion;
use super::tween_motion::{self as motion, STEP_INTERVAL, TWEEN_SECONDS};
use super::tween_settle::{settle, FitContext};
use super::{activity::BusyLease, GlideFrame};

/// A window Glide carries, in the platform's own units: logical points on
/// macOS, physical pixels on Windows. Every call may block on the window's
/// application, so none is ever made with a Glide lock held.
pub(crate) trait GlideWindow: Send + 'static {
  /// A second handle on the same window, for a caller outside the lock the
  /// original is kept behind.
  fn duplicate(&self) -> Self
  where
    Self: Sized;

  /// Whether the window can be moved at all. An application may refuse.
  fn is_movable(&self) -> bool;

  /// Whether the window can be resized, which decides whether a placement may
  /// change its size on the way.
  fn is_resizable(&self) -> bool;

  /// The window's frame as it stands.
  fn frame(&self) -> Option<GlideFrame>;

  /// Readies the window to be moved and reads the frame it starts from. A
  /// window in a state that cannot be moved, maximized on Windows, leaves it
  /// here.
  fn prepare_for_move(&mut self) -> Option<GlideFrame> {
    self.frame()
  }

  /// Writes one frame: its origin, and its size when `resizes`. `settling`
  /// marks the last frame, which a platform may write more carefully.
  /// Answers whether the window took it.
  fn place(&mut self, frame: GlideFrame, resizes: bool, settling: bool) -> bool;

  /// Moves the window to `frame`'s origin, leaving its size alone.
  fn move_to(&mut self, frame: GlideFrame) -> bool;

  /// Brings the window to the front of its application and the screen.
  fn raise(&self);

  fn minimize(&mut self);

  /// Puts the window back where a session found it, at `original`.
  fn restore(&self, original: GlideFrame)
  where
    Self: Sized,
  {
    animate_to(self.duplicate(), original, None);
  }
}

struct Tween {
  target: Box<dyn GlideWindow>,
  /// Where the window was when this tween took over, read once so the curve
  /// starts from the real frame rather than the last one we asked for.
  start: GlideFrame,
  /// Where the window is being carried. For a window its application refuses
  /// to resize this is the region rectangle reduced to a move: the size the
  /// window already has, placed at the region's gravity.
  destination: GlideFrame,
  /// The rectangle the placement asked for, which is what the settling step
  /// judges the achieved frame against - a window that could not take the
  /// region's size must still be reported as not fitting it.
  requested: GlideFrame,
  started_at: Instant,
  /// Whether the window changes size on the way. A pure move leaves the size
  /// alone, so a window that refuses to be resized still travels.
  resizes: bool,
  generation: u64,
  /// What the settling step needs to place a window its application refused
  /// to resize, and to report the frame it ended up with. A move that belongs
  /// to no region carries none.
  fit: Option<FitContext>,
  _busy: BusyLease,
  _completion: completion::Ticket,
}

/// The one tween in flight, if any. Never held across a window call: the
/// input hooks take this lock too, and a blocked hook freezes the pointer.
static TWEEN: Mutex<Option<Tween>> = Mutex::new(None);
/// Bumped by every retarget and cancel, so a step that was already applying a
/// frame drops its result instead of fighting what replaced it.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub(crate) fn start() {
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
/// context the settling step corrects and reports against; a move that is not
/// a region placement passes `None`. A window its application refuses to
/// resize keeps its size and travels anyway, as long as there is a region to
/// align it inside.
pub(crate) fn animate_to(
  mut target: impl GlideWindow,
  destination: GlideFrame,
  fit: Option<FitContext>,
) {
  if !target.is_movable() {
    eprintln!("The application does not allow moving this window");
    return;
  }
  let Some(start) = target.prepare_for_move() else {
    eprintln!("Could not read the window to move");
    return;
  };
  let resizable = target.is_resizable();
  let (travel, resizes) = match fit.as_ref() {
    Some(context) => motion::travel(start, destination, resizable, context.gravity),
    None => {
      let resizes = start.width != destination.width || start.height != destination.height;
      if resizes && !resizable {
        eprintln!("The application does not allow resizing this window");
        return;
      }
      (destination, resizes)
    }
  };
  let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
  if let Ok(mut slot) = TWEEN.lock() {
    *slot = Some(Tween {
      target: Box::new(target),
      start,
      destination: travel,
      requested: destination,
      started_at: Instant::now(),
      resizes,
      generation,
      fit,
      _busy: BusyLease::acquire(),
      _completion: completion::begin(),
    });
  }
}

/// Drops whatever is in flight without moving the window any further. A
/// minimize or a snap back to maximized that follows must not be fought by a
/// late frame.
pub(crate) fn cancel() {
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
    motion::interpolate(tween.start, tween.destination, motion::eased(progress))
  };

  if !is_current(tween.generation) || !tween.target.place(frame, tween.resizes, settling) {
    return;
  }
  if settling {
    // Still outside the lock, and still guarded: reading the window back and
    // correcting it is more window traffic, which a retarget may overtake at
    // any point.
    if let Some(context) = tween.fit.take() {
      let generation = tween.generation;
      settle(
        tween.target.as_mut(),
        tween.requested,
        || is_current(generation),
        &context,
      );
    }
    return;
  }
  if let Ok(mut slot) = TWEEN.lock() {
    if slot.is_none() && is_current(tween.generation) {
      *slot = Some(tween);
    }
  }
}

/// Whether a tween is still the one in flight. Every write a step makes is
/// preceded by this: a retarget that landed meanwhile owns the window now.
fn is_current(generation: u64) -> bool {
  GENERATION.load(Ordering::Relaxed) == generation
}
