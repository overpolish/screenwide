// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a dictation listens with, and the helper kept between dictations
//! with the model files it has loaded, so the next one knows whether it
//! loads them first.

use std::path::PathBuf;

use screenwide_speech::runner::Transcriber;

/// What a dictation listens with.
pub struct Setup {
  /// The model's file.
  pub model: PathBuf,
  /// The microphone's id, or none for the system's default, which also
  /// stands in for one that is gone.
  pub microphone: Option<String>,
  /// Names and terms to write where the audio fits them better than what
  /// was heard.
  pub words: Vec<String>,
  /// How words are written at each pause: by a model that does not stream,
  /// or again after the streaming model.
  pub at_pauses: Option<AtPauses>,
}

/// How a dictation is written at each pause.
pub struct AtPauses {
  /// The voice activity model's file, which finds the pauses.
  pub vad: PathBuf,
  /// A language code such as `fr`, or none to detect it.
  pub language: Option<String>,
  /// The file of the model that writes each stretch again, the setup's
  /// model streaming the words as they are spoken until it has; none for
  /// the setup's model to write each stretch itself.
  pub rewriter: Option<PathBuf>,
}

impl Setup {
  /// The model files the helper loads for this setup.
  pub(super) fn files(&self) -> Vec<PathBuf> {
    let mut files = vec![self.model.clone()];
    if let Some(at_pauses) = &self.at_pauses {
      files.push(at_pauses.vad.clone());
      files.extend(at_pauses.rewriter.clone());
    }
    files
  }
}

/// Dictation's helper, kept for the next dictation, and the model files it
/// has loaded.
pub struct Helper {
  pub(super) transcriber: Transcriber,
  pub(super) loaded: Vec<PathBuf>,
}
