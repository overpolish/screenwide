// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One tool's part in readying the microphone: where its progress is told,
//! and the flag its Cancel sets.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub(crate) trait Step {
  /// Tells how far the tool's work has got, 0 to 1.
  fn tell(&mut self, fraction: f32);
  /// Set once the tool's work is cancelled; `None` for work nobody can stop.
  fn stop(&self) -> Option<Arc<AtomicBool>>;
}

/// A step nobody watches or stops: an export's, which waits on it.
pub(crate) struct Unwatched;

impl Step for Unwatched {
  fn tell(&mut self, _fraction: f32) {}

  fn stop(&self) -> Option<Arc<AtomicBool>> {
    None
  }
}

/// Runs `work` as `step`'s, telling it how far `work` has got, and stopping
/// once its flag is set (`screenwide_speech::stop`).
pub(super) fn run<T>(step: &mut dyn Step, work: impl FnOnce(&mut dyn FnMut(f32)) -> T) -> T {
  let stop = step.stop();
  screenwide_speech::stop::stoppable(stop, || work(&mut |fraction| step.tell(fraction)))
}
