// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's microphone written out as text, word by word: the
//! stretches of speech planned from the listen Remove silences already made
//! (`plan`), and the words' edges settled against the speech and the sound.

pub mod plan;
pub mod word_edges;
pub mod word_ends;
pub mod word_starts;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A word said in the recording, with when it was said.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTranscriptWord {
  pub start_ms: u64,
  pub end_ms: u64,
  pub text: String,
  /// How sure the engine was of it, 0 to 1: low ones are worth a second look
  /// when the text is edited.
  pub probability: f32,
}

/// The recording's microphone as text, in the order it was said.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTranscript {
  pub language: String,
  pub model: String,
  pub words: Vec<RecordingTranscriptWord>,
}
