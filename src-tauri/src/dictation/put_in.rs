// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A finished dictation put where the caret is: its commands carried out,
//! its corrections made, its word list changes made, and its text pasted
//! into the app dictated into. The text stays on the clipboard, so a paste
//! that went nowhere loses nothing.

use screenwide_dictation::{shape, DictatedText, Phrases};
use screenwide_paste::{Pasted, Refusal, Target};
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
  match screenwide_paste::paste(&text, target, &on_main) {
    Ok(Pasted::Typed) => Ok(taught),
    Ok(Pasted::OnClipboard) => Err(DictationProblem::OnClipboard),
    Err(Refusal::SecureField) => Err(DictationProblem::PasswordField),
    Err(Refusal::Elevated) => Err(DictationProblem::Administrator),
    Err(Refusal::Failed(error)) => {
      eprintln!("Could not paste the dictation: {error}");
      Err(DictationProblem::Failed)
    }
  }
}
