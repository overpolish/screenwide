// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Taking notes from a call: begun, fed the microphone and the computer's
//! audio a little at a time, and finished, the lines written coming back
//! each step and the voices told apart at the end.

use std::path::PathBuf;

use screenwide_transcriber::{Job, Reply};
pub use screenwide_transcriber::{NoteLine, NoteSource, NoteWord, SpeakerTurn, VoiceClip};

use super::Transcriber;

/// What notes are taken with.
pub struct NotesModels {
  /// The model that writes each stretch of speech.
  pub model: PathBuf,
  /// The voice activity model, which finds the pauses.
  pub vad: PathBuf,
  /// The speaker model, which tells the voices apart.
  pub speakers: PathBuf,
  /// The voice model, whose fingerprints join a crowded call's voices.
  pub voices: PathBuf,
}

/// The notes' last lines, when each voice in the computer's audio spoke,
/// and a clip of each voice to tell who it is by.
pub struct FinishedNotes {
  pub lines: Vec<NoteLine>,
  pub turns: Vec<SpeakerTurn>,
  pub voices: Vec<VoiceClip>,
}

impl Transcriber {
  /// Starts taking notes with `models`, in `language` or detecting it,
  /// writing `words` where the audio fits them better than what was heard.
  /// Until [`Self::finish_notes`] the helper takes nothing else.
  pub fn begin_notes(
    &mut self,
    NotesModels {
      model,
      vad,
      speakers,
      voices,
    }: NotesModels,
    language: Option<String>,
    words: Vec<String>,
  ) -> Result<(), String> {
    self
      .noted(Job::BeginNotes {
        model,
        vad,
        speakers,
        voices,
        language,
        words,
      })
      .map(|_| ())
  }

  /// The lines written since the last call, once `samples` of `source`,
  /// 16-bit at the protocol's rate starting `at_ms` into the notes, are
  /// heard too.
  pub fn feed_notes(
    &mut self,
    source: NoteSource,
    at_ms: u64,
    samples: Vec<i16>,
  ) -> Result<Vec<NoteLine>, String> {
    self.noted(Job::FeedNotes {
      source,
      at_ms,
      samples,
    })
  }

  /// The notes' last lines, once what the helper still holds is written, and
  /// the voices told apart over the whole call.
  pub fn finish_notes(&mut self) -> Result<FinishedNotes, String> {
    match self.ask(Job::FinishNotes, &mut |_| {})? {
      Reply::NotesFinished {
        lines,
        turns,
        voices,
        ..
      } => Ok(FinishedNotes {
        lines,
        turns,
        voices,
      }),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  fn noted(&mut self, job: Job) -> Result<Vec<NoteLine>, String> {
    match self.ask(job, &mut |_| {})? {
      Reply::Noted { lines, .. } => Ok(lines),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
