// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The profanity and fillers the Transcript panel marks, found by the same
//! lists the captions hide them by: `screenwide_lexicon`.

use std::collections::HashSet;

use screenwide_lexicon::{find_fillers, find_profanity, Finding, ProfanityLexicon, Unit};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One word as the Transcript panel shows it: a word as heard, or the words
/// a correction covers under its text.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TranscriptMarkWord {
  pub text: String,
  pub start_ms: f64,
  pub end_ms: f64,
  /// Cut out by the edit, so never marked.
  pub removed: bool,
}

/// A run of words, `first` to `last` by index, and the folded text it
/// matched, so every mark alike can be found together.
#[derive(Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TranscriptMark {
  pub folded: String,
  pub first: usize,
  pub last: usize,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TranscriptMarks {
  pub profanity: Vec<TranscriptMark>,
  pub fillers: Vec<TranscriptMark>,
}

fn marks(found: Vec<Finding>) -> Vec<TranscriptMark> {
  found
    .into_iter()
    .map(|finding| TranscriptMark {
      folded: finding.folded,
      first: finding.first,
      last: finding.last,
    })
    .collect()
}

/// The profanity and fillers among `words`, in a transcript in `language`.
/// `allowed` is the folded text marked "Not profanity".
#[tauri::command]
pub fn find_transcript_marks(
  language: Option<String>,
  words: Vec<TranscriptMarkWord>,
  allowed: Vec<String>,
) -> TranscriptMarks {
  let units: Vec<Unit> = words
    .iter()
    .map(|word| Unit {
      text: &word.text,
      start_ms: word.start_ms,
      end_ms: word.end_ms,
      removed: word.removed,
    })
    .collect();
  let language = language.as_deref();
  let allowed: HashSet<String> = allowed.into_iter().collect();
  TranscriptMarks {
    profanity: marks(find_profanity(
      &units,
      &ProfanityLexicon::for_language(language),
      &allowed,
    )),
    fillers: marks(find_fillers(&units, language)),
  }
}
