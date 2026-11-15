// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a dictation listens with: the model for the language transcription
//! is set to, streaming in English and written at each pause otherwise, the
//! voice activity model that finds those pauses, the word list, and the
//! spoken commands of that language.

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

/// The setup for a dictation on `microphone`, or why there can be none.
pub(super) fn setup(
  app: &AppHandle,
  microphone: Option<String>,
) -> Result<(Setup, Spoken), DictationProblem> {
  let language = language::resolved(&language::current());
  let (model, writes) = models::dictation();
  let path = models::is_downloaded(app, model)
    .then(|| models::paths(app, model).ok())
    .flatten()
    .and_then(|paths| paths.into_iter().next())
    .ok_or(DictationProblem::NoModel)?;
  let at_pauses = match writes {
    Writes::AsSpoken => None,
    Writes::AtPauses => Some(AtPauses {
      vad: models::vad_model(app).map_err(|error| {
        eprintln!("Could not dictate: {error}");
        DictationProblem::Failed
      })?,
      language: language.clone(),
    }),
  };
  let spoken = Spoken {
    phrases: phrases(language.as_deref()),
    at_pauses: at_pauses.is_some(),
  };
  let setup = Setup {
    model: path,
    microphone,
    words: words::current(),
    at_pauses,
  };
  Ok((setup, spoken))
}
