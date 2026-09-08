// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Completion callbacks follow the tween through its final fit correction.

use std::sync::{Arc, Mutex};

type Callback = Box<dyn FnOnce() + Send>;

struct Completion(Mutex<Option<Vec<Callback>>>);

impl Completion {
  fn new() -> Self {
    Self(Mutex::new(Some(Vec::new())))
  }

  fn after(&self, callback: Callback) {
    let mut pending = self.0.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(callbacks) = pending.as_mut() {
      callbacks.push(callback);
    } else {
      drop(pending);
      callback();
    }
  }

  fn finish(&self) {
    let callbacks = self
      .0
      .lock()
      .unwrap_or_else(|error| error.into_inner())
      .take();
    for callback in callbacks.into_iter().flatten() {
      callback();
    }
  }
}

static CURRENT: Mutex<Option<Arc<Completion>>> = Mutex::new(None);

pub(super) struct Ticket(Arc<Completion>);

impl Drop for Ticket {
  fn drop(&mut self) {
    self.0.finish();
  }
}

pub(super) fn begin() -> Ticket {
  let completion = Arc::new(Completion::new());
  *CURRENT.lock().unwrap_or_else(|error| error.into_inner()) = Some(completion.clone());
  Ticket(completion)
}

pub(crate) fn after_current(callback: Callback) {
  let current = CURRENT
    .lock()
    .unwrap_or_else(|error| error.into_inner())
    .clone();
  if let Some(current) = current {
    current.after(callback);
  } else {
    callback();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};

  #[test]
  fn callbacks_wait_for_tween_ticket_then_run_once() {
    let completion = Arc::new(Completion::new());
    let ticket = Ticket(completion.clone());
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    completion.after(Box::new(move || {
      observed.fetch_add(1, Ordering::SeqCst);
    }));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    drop(ticket);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    completion.after(Box::new(|| {}));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }
}
