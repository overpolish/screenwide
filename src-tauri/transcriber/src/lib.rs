// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the app and the transcriber say to each other: one JSON object per
//! line, a [`Request`] on the helper's standard input and a [`Reply`] for it
//! on its standard output. The helper runs requests in the order they come
//! and exits when its input closes, so the app owns how long it lives.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Audio is handed over as a file of 32-bit float samples in native byte
/// order, mono, at this rate: what the engine takes, so the helper needs no
/// decoder of its own and the app's FFmpeg does the converting.
pub const SAMPLE_RATE: u32 = 16_000;

/// The rate a voice is cleaned at: the speech enhancer's own, which keeps the
/// full range a microphone records.
pub const CLEAN_SAMPLE_RATE: u32 = 48_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  /// Echoed in the reply.
  pub id: u64,
  #[serde(flatten)]
  pub job: Job,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum Job {
  /// Speech written out as text, answered with [`Reply::Done`].
  Transcribe {
    /// The model file to transcribe with.
    model: PathBuf,
    /// What the transcript names the model as.
    model_id: String,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
    /// A language code such as `en`, or nothing to detect it.
    language: Option<String>,
  },
  /// How likely each moment of the audio is to be speech, answered with
  /// [`Reply::Speech`].
  DetectSpeech {
    /// The voice activity model file.
    model: PathBuf,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
  },
  /// The voice in the audio with everything else taken out, answered with
  /// [`Reply::Cleaned`] once it is written.
  CleanSpeech {
    /// The samples, as [`SAMPLE_RATE`] describes but at
    /// [`CLEAN_SAMPLE_RATE`].
    audio: PathBuf,
    /// Where the cleaned samples go, in the same form, lined up with the
    /// samples in and as long.
    output: PathBuf,
  },
  /// The voice in the audio rebuilt as a studio recording, answered with
  /// [`Reply::Restored`] once it is written.
  RestoreSpeech {
    /// The speech encoder of the restoration model.
    features: PathBuf,
    /// Its vocoder.
    decoder: PathBuf,
    /// The samples, as [`SAMPLE_RATE`] describes.
    audio: PathBuf,
    /// Where the restored samples go, in the same form but at
    /// [`CLEAN_SAMPLE_RATE`], lined up with the samples in.
    output: PathBuf,
    /// The stretches to restore, as `[start, end)` in samples at
    /// [`SAMPLE_RATE`], sorted and apart. Everything else comes out silent.
    stretches: Vec<[u64; 2]>,
  },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum Reply {
  Done {
    id: u64,
    transcript: Transcript,
  },
  Speech {
    id: u64,
    speech: Speech,
  },
  /// The voice is written; `gain` lifts it back to its recorded loudness.
  Cleaned {
    id: u64,
    gain: f32,
  },
  /// The restored voice is written.
  Restored {
    id: u64,
  },
  /// How far a long job has got, 0 to 1. Sent any number of times before the
  /// reply that ends the job, which every other kind is.
  Progress {
    id: u64,
    fraction: f32,
  },
  Failed {
    id: u64,
    message: String,
  },
}

/// The chance that each window of the audio holds speech, 0 to 1, in order.
/// Each window is `window_samples` long at [`SAMPLE_RATE`]; the last may run
/// past the end of the audio.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Speech {
  pub probabilities: Vec<f32>,
  pub window_samples: u32,
}

/// Speech as text, with when each word was said. Times are milliseconds from
/// the start of the audio transcribed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
  /// The language it was transcribed as, by code: the one asked for, or the
  /// one detected.
  pub language: String,
  pub model: String,
  /// Every segment's text, joined.
  pub text: String,
  pub segments: Vec<Segment>,
}

/// A stretch of speech the engine decoded in one go: roughly a sentence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
  pub start_ms: u64,
  pub end_ms: u64,
  pub text: String,
  pub words: Vec<Word>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
  pub start_ms: u64,
  pub end_ms: u64,
  pub text: String,
  /// How sure the engine was of it, 0 to 1: low ones are worth a second
  /// look when the text is edited.
  pub probability: f32,
}
