// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Work on a camera that stops part-way once what it was for is turned off
//! or taken away, rather than running on to the end of a long recording.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// What is being made for a camera.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Work {
  /// Finding the person, for the matte.
  Matte,
  /// Following picked objects through the recording.
  Following,
  /// Laying the followed objects into the matte.
  Composing,
}

static RUNNING: Mutex<Vec<(PathBuf, Work, Arc<AtomicBool>)>> = Mutex::new(Vec::new());

/// Work under way, which can be told to stop until it is dropped.
pub(super) struct Running {
  stop: Arc<AtomicBool>,
}

impl Running {
  /// Set once the work is to stop.
  pub(super) fn stop(&self) -> Arc<AtomicBool> {
    Arc::clone(&self.stop)
  }

  pub(super) fn is_stopped(&self) -> bool {
    self.stop.load(Ordering::Acquire)
  }
}

impl Drop for Running {
  fn drop(&mut self) {
    running().retain(|(_, _, stop)| !Arc::ptr_eq(stop, &self.stop));
  }
}

/// Starts `work` on the camera in `folder`.
pub(super) fn begin(folder: &Path, work: Work) -> Running {
  let stop = Arc::new(AtomicBool::new(false));
  running().push((folder.to_owned(), work, Arc::clone(&stop)));
  Running { stop }
}

/// Stops any `work` under way on the camera in `folder`.
pub(super) fn stop(folder: &Path, work: Work) {
  for (_, _, stop) in running()
    .iter()
    .filter(|(at, each, _)| at == folder && *each == work)
  {
    stop.store(true, Ordering::Release);
  }
}

/// Whether `work` is under way on the camera in `folder`.
pub(super) fn is_running(folder: &Path, work: Work) -> bool {
  running()
    .iter()
    .any(|(at, each, _)| at == folder && *each == work)
}

fn running() -> std::sync::MutexGuard<'static, Vec<(PathBuf, Work, Arc<AtomicBool>)>> {
  RUNNING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}
