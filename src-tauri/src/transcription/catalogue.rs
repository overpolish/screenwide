// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;

/// What a model transcribes. Each use has its own model, chosen for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Purpose {
  /// Voice notes: a few seconds each, wanted soon after a recording stops,
  /// so the quick model.
  Moments,
}

#[derive(Debug)]
pub(crate) struct Model {
  /// The model's own name, as transcripts record it.
  pub id: &'static str,
  /// What it is for, as Settings names it.
  pub name: &'static str,
  /// What downloading it gets you.
  pub description: &'static str,
  pub purpose: Purpose,
  pub file: &'static str,
  pub size_bytes: u64,
  pub sha256: &'static str,
}

/// The whisper.cpp project's own conversions of OpenAI's Whisper models, at
/// a fixed revision: a file that changed upstream would fail its checksum
/// rather than arrive unnoticed.
const REPOSITORY: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve";
const REVISION: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

pub(crate) const MODELS: &[Model] = &[Model {
  description: "Turns your moments' voice notes into text.",
  file: "ggml-base.bin",
  id: "base",
  name: "Moments",
  purpose: Purpose::Moments,
  sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
  size_bytes: 147_951_465,
}];

impl Model {
  pub(crate) fn url(&self) -> String {
    format!("{REPOSITORY}/{REVISION}/{}", self.file)
  }
}

pub(crate) fn find(id: &str) -> Result<&'static Model, String> {
  MODELS
    .iter()
    .find(|model| model.id == id)
    .ok_or_else(|| format!("There is no transcription model called {id}"))
}

/// The model chosen for `purpose`.
pub(crate) fn for_purpose(purpose: Purpose) -> Option<&'static Model> {
  MODELS.iter().find(|model| model.purpose == purpose)
}
