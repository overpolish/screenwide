// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The engine's timers as run-loop timers on the main thread, keyed by the
//! window and the engine's id as Win32 timers are.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::NSTimer;

use super::super::region_osc::host::WindowId;
use super::super::region_osc::input as engine;

thread_local! {
  static TIMERS: RefCell<HashMap<(isize, usize), Retained<NSTimer>>> =
    RefCell::new(HashMap::new());
}

pub(super) fn set(window: WindowId, timer: usize, interval_ms: u32) {
  kill(window, timer);
  let tick = RcBlock::new(move |_timer: NonNull<NSTimer>| {
    let fired = std::panic::catch_unwind(|| engine::timer(window, timer));
    if fired.is_err() {
      eprintln!("The macOS region overlay dropped a timer after a panic");
    }
  });
  let seconds = f64::from(interval_ms.max(1)) / 1_000.0;
  // Safety: the block lives as long as the timer, which is invalidated when
  // killed or when its window goes.
  let scheduled =
    unsafe { NSTimer::scheduledTimerWithTimeInterval_repeats_block(seconds, true, &tick) };
  TIMERS.with(|timers| timers.borrow_mut().insert((window.0, timer), scheduled));
}

pub(super) fn kill(window: WindowId, timer: usize) {
  if let Some(scheduled) = TIMERS.with(|timers| timers.borrow_mut().remove(&(window.0, timer))) {
    scheduled.invalidate();
  }
}

/// Stops every timer of a window going away.
pub(super) fn kill_all(window: WindowId) {
  let stopped: Vec<_> = TIMERS.with(|timers| {
    let mut timers = timers.borrow_mut();
    let keys: Vec<_> = timers
      .keys()
      .filter(|(id, _)| *id == window.0)
      .copied()
      .collect();
    keys
      .into_iter()
      .filter_map(|key| timers.remove(&key))
      .collect()
  });
  for timer in stopped {
    timer.invalidate();
  }
}
