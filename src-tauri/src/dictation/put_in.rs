// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A finished dictation put where the caret is: its commands carried out,
//! its corrections made, its word list changes made, and its text pasted
//! into the app dictated into.

use screenwide_dictation::{shape, DictatedText, Phrases};
use screenwide_paste::{Refusal, Target};
use tauri::AppHandle;

use super::corrections::Corrections;
use super::island::DictationProblem;
use super::taught::{self, DictationWordChange};

/// Pastes what was heard into `target`, once the commands of `phrases` are
/// carried out and the speaker's corrections made. Answers the words added
/// or removed by voice, or why it could not paste.
pub(super) fn put_in(
  app: &AppHandle,
  heard: Result<DictatedText, String>,
  (target, phrases, corrections): (Option<Target>, &Phrases, Option<&Corrections>),
) -> Result<Vec<DictationWordChange>, DictationProblem> {
  let heard = heard.map_err(|error| {
    eprintln!("Dictation failed: {error}");
    DictationProblem::Failed
  })?;
  let shaped = shape(&heard, phrases);
  let taught = taught::apply(app, &shaped.changes);
  let text = match corrections {
    Some(corrections) => corrections.finish(&shaped.text()),
    None => shaped.text(),
  };
  if text.trim().is_empty() {
    return Ok(taught);
  }
  let on_main = |work: Box<dyn FnOnce() + Send>| {
    let _ = app.run_on_main_thread(work);
  };
  screenwide_paste::paste(&text, target, &on_main)
    .map(|()| taught)
    .map_err(|refusal| match refusal {
      Refusal::SecureField => DictationProblem::PasswordField,
      Refusal::Elevated => DictationProblem::Administrator,
      Refusal::Failed(error) => {
        eprintln!("Could not paste the dictation: {error}");
        DictationProblem::Failed
      }
    })
}
