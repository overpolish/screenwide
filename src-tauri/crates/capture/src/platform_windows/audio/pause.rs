// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a recording's audio is paused, and which run between pauses it
//! is in. A pause leaves nothing in the recording, so the sidecar writer
//! keeps lost audio as silence only within a run, never across a pause.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Default)]
pub(super) struct Pause {
  paused: AtomicBool,
  resumes: AtomicU64,
}

impl Pause {
  pub(super) fn pause(&self) {
    self.paused.store(true, Ordering::Release);
  }

  /// Starts the next run before letting audio through, so nothing captured
  /// after the pause is taken for part of the run before it.
  pub(super) fn resume(&self) {
    self.resumes.fetch_add(1, Ordering::AcqRel);
    self.paused.store(false, Ordering::Release);
  }

  /// The run audio captured now belongs to; none while paused.
  pub(super) fn run(&self) -> Option<u64> {
    (!self.paused.load(Ordering::Acquire)).then(|| self.resumes.load(Ordering::Acquire))
  }
}
