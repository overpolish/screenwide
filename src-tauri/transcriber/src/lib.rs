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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  /// Echoed in the reply.
  pub id: u64,
  /// The model file to transcribe with.
  pub model: PathBuf,
  /// What the transcript names the model as.
  pub model_id: String,
  /// The samples, as [`SAMPLE_RATE`] describes.
  pub audio: PathBuf,
  /// A language code such as `en`, or nothing to detect it.
  pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum Reply {
  Done { id: u64, transcript: Transcript },
  Failed { id: u64, message: String },
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
