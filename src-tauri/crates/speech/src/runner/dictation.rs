// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictating: a stream begun, fed the microphone a little at a time, and
//! finished, the text heard so far coming back each step.

use std::path::PathBuf;

pub use screenwide_transcriber::{DictatedText, DictatedWord, SAMPLE_RATE};
use screenwide_transcriber::{Job, Reply};

use super::Transcriber;

impl Transcriber {
  /// Starts a dictation with the streaming model at `model`, writing `words`
  /// where the audio fits them better than what was heard. Until
  /// [`Self::finish_dictation`] the helper takes nothing else.
  pub fn begin_dictation(&mut self, model: PathBuf, words: Vec<String>) -> Result<(), String> {
    self
      .dictated(Job::BeginDictation { model, words })
      .map(|_| ())
  }

  /// Starts a dictation that writes each stretch of speech once a pause ends
  /// it, with the model at `model`, finding the pauses with the voice
  /// activity model at `vad`, in `language`, or detecting it.
  pub fn begin_dictation_at_pauses(
    &mut self,
    (model, vad): (PathBuf, PathBuf),
    language: Option<String>,
    words: Vec<String>,
  ) -> Result<(), String> {
    self
      .dictated(Job::BeginDictationAtPauses {
        model,
        vad,
        language,
        words,
      })
      .map(|_| ())
  }

  /// Starts a dictation that streams with the model at `model` and writes
  /// each stretch of speech again once a pause ends it with the model at
  /// `rewriter`, finding the pauses with the voice activity model at `vad`,
  /// in `language`, or detecting it.
  pub fn begin_dictation_rewritten(
    &mut self,
    (model, rewriter, vad): (PathBuf, PathBuf, PathBuf),
    language: Option<String>,
    words: Vec<String>,
  ) -> Result<(), String> {
    self
      .dictated(Job::BeginDictationRewritten {
        model,
        rewriter,
        vad,
        language,
        words,
      })
      .map(|_| ())
  }

  /// The text heard once `samples`, 16-bit at the protocol's rate, are heard
  /// too.
  pub fn feed_dictation(&mut self, samples: Vec<i16>) -> Result<DictatedText, String> {
    self.dictated(Job::FeedDictation { samples })
  }

  /// All the dictation's text, once what it still holds is heard.
  pub fn finish_dictation(&mut self) -> Result<DictatedText, String> {
    self.dictated(Job::FinishDictation)
  }

  /// `text` as the language model at `model` corrects it, told what to do
  /// by `instructions`, on the cleanup helper.
  pub fn correct_text(
    &mut self,
    model: PathBuf,
    instructions: String,
    text: String,
  ) -> Result<String, String> {
    match self.ask(
      Job::CorrectText {
        model,
        instructions,
        text,
      },
      &mut |_| {},
    )? {
      Reply::Corrected { text, .. } => Ok(text),
      _ => Err("The cleanup helper answered with something else".to_owned()),
    }
  }

  fn dictated(&mut self, job: Job) -> Result<DictatedText, String> {
    match self.ask(job, &mut |_| {})? {
      Reply::Dictated { text, .. } => Ok(text),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
