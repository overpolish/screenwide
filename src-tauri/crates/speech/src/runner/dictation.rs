// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictating: a stream begun, fed the microphone a little at a time, and
//! finished, the text heard so far coming back each step.

use std::path::PathBuf;

pub use screenwide_transcriber::{DictatedText, SAMPLE_RATE};
use screenwide_transcriber::{Job, Reply};

use super::Transcriber;

impl Transcriber {
  /// Starts a dictation with the streaming model at `model`. Until
  /// [`Self::finish_dictation`] the helper takes nothing else.
  pub fn begin_dictation(&mut self, model: PathBuf) -> Result<(), String> {
    self.dictated(Job::BeginDictation { model }).map(|_| ())
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

  fn dictated(&mut self, job: Job) -> Result<DictatedText, String> {
    match self.ask(job, &mut |_| {})? {
      Reply::Dictated { text, .. } => Ok(text),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
