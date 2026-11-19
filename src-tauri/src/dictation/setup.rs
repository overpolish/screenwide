// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a dictation listens with: the model for the language transcription
//! is set to, streaming in English, each stretch then written again once a
//! pause ends it, and written at each pause otherwise; the voice activity
//! model that finds those pauses, the word list, the spoken commands of that
//! language, and in English, once downloaded, the model that corrects what
//! the speaker corrected as they spoke.

use screenwide_dictation::correct::{CleanupModel, SPEAKOFLOW_MINI};
use screenwide_dictation::{phrases, AtPauses, Phrases, Setup};
use screenwide_speech::catalogue::Writes;
use tauri::AppHandle;

use super::island::DictationProblem;
use crate::transcription::{language, models, words};

/// How what is heard is told: the language's commands, and whether the
/// text comes at each pause.
#[derive(Clone, Copy)]
pub(super) struct Spoken {
  pub phrases: &'static Phrases,
  pub at_pauses: bool,
}

/// What another listener on the microphone hears with: dictation's model
/// for the language transcription is set to, on the island's microphone,
/// or why there is none (`DictationProblem::NoModel`). The teleprompter
/// follows the voice with it.
pub(crate) fn listening_setup(app: &AppHandle) -> Result<Setup, DictationProblem> {
  let microphone = super::with(|dictation| dictation.microphone.clone());
  setup(app, microphone).map(|(setup, ..)| setup)
}

/// The setup for a dictation on `microphone`, and its cleanup model if it
/// has one, or why there can be none.
pub(super) fn setup(
  app: &AppHandle,
  microphone: Option<String>,
) -> Result<(Setup, Spoken, Option<CleanupModel>), DictationProblem> {
  let language = language::resolved(&language::current());
  let (model, writes) = models::dictation();
  let mut paths = models::is_downloaded(app, model)
    .then(|| models::paths(app, model).ok())
    .flatten()
    .ok_or(DictationProblem::NoModel)?
    .into_iter();
  let path = paths.next().ok_or(DictationProblem::NoModel)?;
  // A streaming model's second file is the model that writes each stretch
  // again once a pause ends it.
  let rewriter = paths.next().filter(|_| writes == Writes::AsSpoken);
  let at_pauses = if writes == Writes::AtPauses || rewriter.is_some() {
    Some(AtPauses {
      vad: models::vad_model(app).map_err(|error| {
        eprintln!("Could not dictate: {error}");
        DictationProblem::Failed
      })?,
      language: language.clone(),
      rewriter,
    })
  } else {
    None
  };
  let spoken = Spoken {
    phrases: phrases(language.as_deref()),
    at_pauses: writes == Writes::AtPauses,
  };
  let setup = Setup {
    model: path,
    microphone,
    words: words::current(),
    at_pauses,
  };
  let cleanup = models::dictation_cleanup()
    .filter(|model| models::is_downloaded(app, model))
    .and_then(|model| models::paths(app, model).ok())
    .and_then(|paths| paths.into_iter().next())
    .map(|path| CleanupModel {
      path,
      prompt: SPEAKOFLOW_MINI,
    });
  Ok((setup, spoken, cleanup))
}
