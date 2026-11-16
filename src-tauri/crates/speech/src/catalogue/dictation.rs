// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which models dictation takes, by language.

use serde::Serialize;

use super::{for_purpose, Model, Purpose, MODELS};

/// When dictation's words appear.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Writes {
  /// As they are spoken: the model streams.
  AsSpoken,
  /// Each stretch of speech once a pause ends it.
  AtPauses,
}

/// What Settings says dictation's model does, by when its words appear and
/// whether it is the Transcripts model, downloaded once for both.
pub fn dictation_description(writes: Writes, shared: bool) -> String {
  match (writes, shared) {
    (Writes::AsSpoken, _) => screenwide_i18n::t!("settings-transcription-dictation-description"),
    (Writes::AtPauses, false) => {
      screenwide_i18n::t!("settings-transcription-dictation-at-pauses-description")
    }
    (Writes::AtPauses, true) => {
      screenwide_i18n::t!("settings-transcription-dictation-shared-description")
    }
  }
}

/// The languages Parakeet Ultra dictates, other than English, which streams,
/// and Greek, which Whisper writes with under half the mistakes (FLEURS).
const ULTRA_LANGUAGES: [&str; 23] = [
  "bg", "cs", "da", "de", "es", "et", "fi", "fr", "hr", "hu", "it", "lt", "lv", "mt", "nl", "pl",
  "pt", "ro", "ru", "sk", "sl", "sv", "uk",
];

/// The model dictation takes for `language`, a code such as `fr`, or none
/// to detect it, and when its words appear: English streams; Parakeet
/// Ultra's languages take it; every other language, and one to be detected,
/// takes the Transcripts model, Whisper, which writes them all.
pub fn dictation(language: Option<&str>) -> (&'static Model, Writes) {
  let by_id = |id: &str| {
    MODELS
      .iter()
      .find(|model| model.id == id)
      .expect("the catalogue has every dictation model")
  };
  match language {
    Some("en") => (by_id("parakeet-unified-en-0.6b-q8_0"), Writes::AsSpoken),
    Some(code) if ULTRA_LANGUAGES.contains(&code) => {
      (by_id("parakeet-ultra-q8_0"), Writes::AtPauses)
    }
    _ => (by_id("large-v3-turbo-q8_0"), Writes::AtPauses),
  }
}

/// The model that corrects dictation in `language`: English only, the one
/// language the cleanup model was trained and scored on.
pub fn dictation_cleanup(language: Option<&str>) -> Option<&'static Model> {
  (language == Some("en"))
    .then(|| for_purpose(Purpose::DictationCleanup))
    .flatten()
}
