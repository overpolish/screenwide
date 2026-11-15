// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Words added to or removed from the word list by "add word" and "remove
//! word" while dictating. A dictation's commands come as everything said
//! so far, so each is carried out once, as it first settles, and what came
//! of each is kept for the island to confirm.

use std::sync::{Mutex, PoisonError};

use screenwide_dictation::WordChange;
use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

use crate::transcription::words;

/// A word changed by voice in this dictation, and what came of it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DictationWordChange {
  pub word: String,
  pub outcome: WordOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WordOutcome {
  Added,
  /// Asked to be added, and listed already.
  AlreadyListed,
  Removed,
  /// Asked to be removed, and not listed.
  NotListed,
  /// The list could not be saved.
  Failed,
}

/// What this dictation's commands came to, in the order they were said.
static TAUGHT: Mutex<Vec<DictationWordChange>> = Mutex::new(Vec::new());

/// A dictation starts with nothing taught.
pub(super) fn reset() {
  TAUGHT
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .clear();
}

/// Carries out the changes in `said` not yet carried out, and answers what
/// all of this dictation's came to.
pub(super) fn apply(app: &AppHandle, said: &[WordChange]) -> Vec<DictationWordChange> {
  let mut taught = TAUGHT.lock().unwrap_or_else(PoisonError::into_inner);
  for change in said.iter().skip(taught.len()) {
    let (word, result, done, undone) = match change {
      WordChange::Add(word) => (
        word,
        words::add(app, word),
        WordOutcome::Added,
        WordOutcome::AlreadyListed,
      ),
      WordChange::Remove(word) => (
        word,
        words::remove(app, word),
        WordOutcome::Removed,
        WordOutcome::NotListed,
      ),
    };
    let outcome = match result {
      Ok(true) => done,
      Ok(false) => undone,
      Err(error) => {
        eprintln!("Could not change the word list: {error}");
        WordOutcome::Failed
      }
    };
    taught.push(DictationWordChange {
      word: word.clone(),
      outcome,
    });
  }
  taught.clone()
}
