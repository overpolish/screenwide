// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focused control, read on a thread of its own. UI Automation asks the
//! focused window's process, which can take seconds to answer or never does,
//! and the keys must not wait for it: a recorder stuck behind the question
//! records nothing more and never lets its hook go.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::super::FocusContext;
use super::classifier;

/// How often the focus is read.
const EVERY: Duration = Duration::from_millis(25);
/// How old the last answer may be and still be trusted.
const STALE: Duration = Duration::from_millis(250);

pub(super) struct FocusProbe {
  focus: Arc<AtomicU8>,
  /// When the last answer came, in milliseconds after `origin`.
  answered_ms: Arc<AtomicU64>,
  origin: Instant,
  stop: Arc<AtomicBool>,
}

impl FocusProbe {
  /// Reads the focus into `focus`, which the hook also reads, until dropped.
  pub(super) fn start(focus: Arc<AtomicU8>) -> Self {
    let probe = Self {
      focus,
      answered_ms: Arc::new(AtomicU64::new(0)),
      origin: Instant::now(),
      stop: Arc::new(AtomicBool::new(false)),
    };
    let (focus, answered_ms, origin, stop) = (
      Arc::clone(&probe.focus),
      Arc::clone(&probe.answered_ms),
      probe.origin,
      Arc::clone(&probe.stop),
    );
    // Never joined: a question that never comes back holds only this thread.
    let _ = std::thread::Builder::new()
      .name("screenwide-keyboard-focus".to_owned())
      .spawn(move || {
        let source = classifier::Focus::enter();
        while !stop.load(Ordering::Acquire) {
          let context = source.context();
          focus.store(classifier::encode_focus(context), Ordering::Release);
          answered_ms.store(origin.elapsed().as_millis() as u64, Ordering::Release);
          std::thread::sleep(EVERY);
        }
      });
    probe
  }

  /// The focus as last read, or unknown where that answer is too old.
  pub(super) fn current(&self) -> FocusContext {
    let age = (self.origin.elapsed().as_millis() as u64)
      .saturating_sub(self.answered_ms.load(Ordering::Acquire));
    if age > STALE.as_millis() as u64 {
      FocusContext::Unknown
    } else {
      classifier::decode_focus(self.focus.load(Ordering::Acquire))
    }
  }
}

impl Drop for FocusProbe {
  fn drop(&mut self) {
    self.stop.store(true, Ordering::Release);
  }
}
