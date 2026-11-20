// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Work a person can stop part-way. The app runs a tool's work inside
//! [`stoppable`] with the flag its Cancel sets, and the work stops at the
//! next point that looks: the helper program is ended as its next reply
//! arrives, and the long loops here look once a chunk. The flag is the
//! thread's, so a tool's work needs no flag passed down through every call
//! it makes; work stopped answers [`STOPPED`].

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What stopped work answers with.
pub const STOPPED: &str = "The work was stopped";

thread_local! {
  static FLAG: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
}

/// Runs `work` on this thread, stopping where it looks once `flag` is set.
/// Without a flag it runs to its end. The flag before is put back after.
pub fn stoppable<T>(flag: Option<Arc<AtomicBool>>, work: impl FnOnce() -> T) -> T {
  let Some(flag) = flag else {
    return work();
  };
  let before = FLAG.with(|current| current.replace(Some(flag)));
  let done = work();
  FLAG.with(|current| current.replace(before));
  done
}

/// Whether `error` is work stopped rather than failed.
pub fn is_stopped(error: &str) -> bool {
  error == STOPPED
}

/// The flag the work on this thread stops by, if it can be stopped.
pub(crate) fn flag() -> Option<Arc<AtomicBool>> {
  FLAG.with(|current| current.borrow().clone())
}

/// Whether the work on this thread has been asked to stop.
pub(crate) fn requested() -> bool {
  FLAG.with(|current| {
    current
      .borrow()
      .as_ref()
      .is_some_and(|flag| flag.load(Ordering::Acquire))
  })
}
