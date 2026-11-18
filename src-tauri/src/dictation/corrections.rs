// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Corrections, wired into dictation: at each pause what the speaker
//! corrected as they spoke is put right on the island while they go on
//! (`screenwide_dictation::correct`), and when they stop the paste waits up
//! to a second for the rest, pasting it as heard if it does not come.
//!
//! The cleanup helper is kept for a few minutes after each dictation, with
//! its model loaded, as dictation's helper is.

use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::{Duration, Instant};

use screenwide_dictation::correct::{CleanupModel, Corrector};
use screenwide_dictation::{paused, DictatedText};
use screenwide_speech::runner::Transcriber;
use tauri::AppHandle;

use super::island::{self, DictationIslandState};

/// How long correcting may hold up the paste once dictation stops.
const BUDGET: Duration = Duration::from_secs(1);
/// How long the cleanup helper is kept after a dictation.
const KEEP_HELPER: Duration = Duration::from_secs(180);

/// The kept helper, and how many dictations have taken it, for the timer
/// that ends it.
static HELPER: Mutex<(Option<Transcriber>, u64)> = Mutex::new((None, 0));

pub(super) struct Corrections {
  corrector: Corrector,
  /// What the island was last told, as heard, to show again corrected when
  /// a correction comes back.
  last: Mutex<Option<DictationIslandState>>,
}

impl Corrections {
  /// Starts correcting a dictation with `model`, its corrections shown on
  /// the island as they come back.
  pub fn start(app: &AppHandle, model: CleanupModel) -> Arc<Self> {
    let (helper, used) = {
      let mut kept = HELPER.lock().unwrap_or_else(PoisonError::into_inner);
      kept.1 += 1;
      (kept.0.take(), kept.1)
    };
    let app = app.clone();
    Arc::new_cyclic(|this: &Weak<Self>| {
      let this = this.clone();
      let corrected = move || {
        if let Some(this) = this.upgrade() {
          this.show_again(&app);
        }
      };
      Self {
        corrector: Corrector::start(model, helper, corrected, move |helper| keep(helper, used)),
        last: Mutex::new(None),
      }
    })
  }

  /// `state`, the island's for `heard`, with the corrections made so far,
  /// once what is open is set to be corrected if the speaker has paused.
  pub fn heard(&self, heard: &DictatedText, state: DictationIslandState) -> DictationIslandState {
    if let DictationIslandState::Listening {
      settled, unsettled, ..
    } = &state
    {
      if paused(heard) {
        self
          .corrector
          .paused(&format!("{settled}{unsettled}"), settled.len());
      }
    }
    *self.last.lock().unwrap_or_else(PoisonError::into_inner) = Some(state.clone());
    self.corrected(state)
  }

  /// `text`, the whole dictation, corrected within the budget.
  pub fn finish(&self, text: &str) -> String {
    self.corrector.finish(text, Instant::now() + BUDGET)
  }

  fn corrected(&self, mut state: DictationIslandState) -> DictationIslandState {
    if let DictationIslandState::Listening { settled, .. } = &mut state {
      *settled = self.corrector.shown(settled);
    }
    state
  }

  fn show_again(&self, app: &AppHandle) {
    let last = self
      .last
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .clone();
    if let Some(state) = last.filter(|_| super::is_listening()) {
      island::tell(app, self.corrected(state));
    }
  }
}

/// Keeps the helper for the next dictation, and ends it once none has come
/// for `KEEP_HELPER`, so its model's memory is given back.
fn keep(helper: Option<Transcriber>, used: u64) {
  HELPER.lock().unwrap_or_else(PoisonError::into_inner).0 = helper;
  std::thread::spawn(move || {
    std::thread::sleep(KEEP_HELPER);
    let idle = {
      let mut kept = HELPER.lock().unwrap_or_else(PoisonError::into_inner);
      (kept.1 == used).then(|| kept.0.take()).flatten()
    };
    drop(idle);
  });
}
