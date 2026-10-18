// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! AppKit's views belong to the main thread, and the engine calls its host
//! from whichever thread a frame or an input arrived on.

use std::sync::Arc;

use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::MainThreadMarker;

/// Main-thread-only `T`, shareable with the threads the engine calls from.
pub(super) struct OnMain<T: 'static>(Arc<MainThreadBound<T>>);

impl<T: 'static> OnMain<T> {
  pub(super) fn new(value: T, mtm: MainThreadMarker) -> Self {
    Self(Arc::new(MainThreadBound::new(value, mtm)))
  }

  /// Runs `work` with the value on the main thread's next turn, even when
  /// called from it: a view change can call back into the engine (a hidden
  /// first responder resigns, and the typing hears of it), and the caller
  /// may hold the engine's state.
  pub(super) fn later(&self, work: impl FnOnce(&T, MainThreadMarker) + Send + 'static) {
    let bound = Arc::clone(&self.0);
    DispatchQueue::main().exec_async(move || {
      // Safety: the main queue runs its blocks on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };
      work(bound.get(mtm), mtm);
    });
  }
}
