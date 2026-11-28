// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A part of the island whose state is one value: what it last told the
//! page, sent on its own event as it changes, and its claim on the island,
//! held while there is a value and let go with it. Keeping both in one
//! place means a part cannot clear what it shows and still hold the island,
//! or let the island go while it still shows something.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::super::island_placement::IslandPlacement;
use super::claims::{self, IslandClaim};
use crate::app_windows::WindowLabel;

pub(crate) struct IslandSlot<T> {
  claim: IslandClaim,
  /// The event the page hears the value on: the value, or `null` once the
  /// part has gone.
  event: &'static str,
  shown: Mutex<Option<T>>,
}

impl<T: Clone + PartialEq + Serialize + Send + 'static> IslandSlot<T> {
  pub(crate) const fn new(claim: IslandClaim, event: &'static str) -> Self {
    Self {
      claim,
      event,
      shown: Mutex::new(None),
    }
  }

  fn shown(&self) -> MutexGuard<'_, Option<T>> {
    self.shown.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// What the page was last told; none while the part is not on the island.
  /// Its command for a page that loads late answers with this.
  pub(crate) fn current(&self) -> Option<T> {
    self.shown().clone()
  }

  pub(crate) fn is_up(&self) -> bool {
    self.shown().is_some()
  }

  /// The page shows `state`, and the island comes out at the top for it if
  /// this part was not on it. Only a change is sent.
  pub(crate) fn tell(&self, app: &AppHandle, state: T) {
    if self.set(app, state) {
      claims::claim(app, self.claim);
    }
  }

  /// Changes what the page shows, only while this part is on the island: a
  /// change that arrives after it went does not bring it back.
  pub(crate) fn update(&self, app: &AppHandle, state: T) {
    {
      let mut shown = self.shown();
      if shown.is_none() || shown.as_ref() == Some(&state) {
        return;
      }
      *shown = Some(state.clone());
    }
    self.send(app, Some(state));
  }

  /// As `tell`, hanging the island at `placement` while this part is on top.
  pub(crate) fn tell_at(&self, app: &AppHandle, state: T, placement: IslandPlacement) {
    self.set(app, state);
    claims::claim_at(app, self.claim, placement);
  }

  /// Shows `state` for `shown`, then withdraws, unless something else was
  /// shown meanwhile.
  pub(crate) fn tell_for(&'static self, app: &AppHandle, state: T, shown: Duration) {
    self.tell(app, state.clone());
    let app = app.clone();
    std::thread::spawn(move || {
      std::thread::sleep(shown);
      if self.current().as_ref() == Some(&state) {
        self.withdraw(&app);
      }
    });
  }

  /// The part is done: the page is told, and the island goes back to
  /// whatever else holds it, or tucks away. Returns whether it was up.
  pub(crate) fn withdraw(&self, app: &AppHandle) -> bool {
    if self.shown().take().is_none() {
      return false;
    }
    self.send(app, None);
    claims::release(app, self.claim);
    true
  }

  /// Sets and sends `state`; returns whether the part was not up before.
  fn set(&self, app: &AppHandle, state: T) -> bool {
    let first = {
      let mut shown = self.shown();
      if shown.as_ref() == Some(&state) {
        return false;
      }
      let first = shown.is_none();
      *shown = Some(state.clone());
      first
    };
    self.send(app, Some(state));
    first
  }

  // Advisory: what the part does never fails because the page did not hear.
  fn send(&self, app: &AppHandle, state: Option<T>) {
    let _ = app.emit_to(WindowLabel::Island.as_str(), self.event, state);
  }
}
