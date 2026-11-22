// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focused control, read on a thread of its own. UI Automation asks the
//! focused window's process, which can take seconds to answer or never does,
//! and the keys must not wait for it: a recorder stuck behind the question
//! records nothing more and never lets its hook go.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::super::{CaretRect, FocusContext};
use super::classifier;

/// How often the focus is read.
const EVERY: Duration = Duration::from_millis(25);
/// How old the last answer may be and still be trusted.
const STALE: Duration = Duration::from_millis(250);
/// How often the caret is read while a text control has the keyboard: the
/// writer keeps it a few times a second.
const CARET_EVERY: Duration = Duration::from_millis(100);

/// The caret as last read, and when.
type CaretReading = Arc<Mutex<Option<(Instant, CaretRect)>>>;

pub(super) struct FocusProbe {
  focus: Arc<AtomicU8>,
  /// When the last answer came, in milliseconds after `origin`.
  answered_ms: Arc<AtomicU64>,
  origin: Instant,
  caret: CaretReading,
  stop: Arc<AtomicBool>,
}

impl FocusProbe {
  /// Reads the focus into `focus`, which the hook also reads, until dropped.
  pub(super) fn start(focus: Arc<AtomicU8>) -> Self {
    let probe = Self {
      focus,
      answered_ms: Arc::new(AtomicU64::new(0)),
      origin: Instant::now(),
      caret: Arc::new(Mutex::new(None)),
      stop: Arc::new(AtomicBool::new(false)),
    };
    let (focus, answered_ms, origin, caret, stop) = (
      Arc::clone(&probe.focus),
      Arc::clone(&probe.answered_ms),
      probe.origin,
      Arc::clone(&probe.caret),
      Arc::clone(&probe.stop),
    );
    // Never joined: a question that never comes back holds only this thread.
    let _ = std::thread::Builder::new()
      .name("screenwide-keyboard-focus".to_owned())
      .spawn(move || {
        let source = classifier::Focus::enter();
        let mut caret_read: Option<Instant> = None;
        while !stop.load(Ordering::Acquire) {
          let context = source.context();
          focus.store(classifier::encode_focus(context), Ordering::Release);
          answered_ms.store(origin.elapsed().as_millis() as u64, Ordering::Release);
          if context == FocusContext::Text
            && caret_read.is_none_or(|at| at.elapsed() >= CARET_EVERY)
          {
            caret_read = Some(Instant::now());
            let read = source.caret().map(|rect| (Instant::now(), rect));
            *caret.lock().unwrap_or_else(PoisonError::into_inner) = read;
          }
          std::thread::sleep(EVERY);
        }
      });
    probe
  }

  /// The caret as last read, where that answer is fresh enough.
  pub(super) fn caret(&self) -> Option<CaretRect> {
    let reading = *self.caret.lock().unwrap_or_else(PoisonError::into_inner);
    reading
      .filter(|(at, _)| at.elapsed() <= STALE)
      .map(|(_, rect)| rect)
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
