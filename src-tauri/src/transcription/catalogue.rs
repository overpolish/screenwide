// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;

/// What a model is for. Each use has its own model, chosen for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Purpose {
  /// Voice notes: a few seconds each, wanted soon after a recording stops,
  /// so the quick model.
  Moments,
  /// Studio sound: the microphone rebuilt as a studio recording.
  StudioSound,
}

impl Purpose {
  /// What Settings calls the model for this use.
  pub(crate) fn name(self) -> String {
    match self {
      Self::Moments => crate::i18n::t!("settings-transcription-moments"),
      Self::StudioSound => crate::i18n::t!("settings-transcription-studio-sound"),
    }
  }

  /// What downloading the model for this use gets you.
  pub(crate) fn description(self) -> String {
    match self {
      Self::Moments => crate::i18n::t!("settings-transcription-moments-description"),
      Self::StudioSound => crate::i18n::t!("settings-transcription-studio-sound-description"),
    }
  }

  /// Whether the model turns speech into text, so one transcribing model can
  /// stand in for another.
  pub(crate) fn transcribes(self) -> bool {
    match self {
      Self::Moments => true,
      Self::StudioSound => false,
    }
  }
}

/// One file of a model, fetched from a fixed address: a file that changed
/// there would fail its checksum rather than arrive unnoticed.
#[derive(Debug)]
pub(crate) struct ModelFile {
  pub name: &'static str,
  pub url: &'static str,
  pub size_bytes: u64,
  pub sha256: &'static str,
}

#[derive(Debug)]
pub(crate) struct Model {
  /// The model's own name, as transcripts record it.
  pub id: &'static str,
  /// What it is for, which also names it in Settings.
  pub purpose: Purpose,
  pub files: &'static [ModelFile],
}

impl Model {
  pub(crate) fn size_bytes(&self) -> u64 {
    self.files.iter().map(|file| file.size_bytes).sum()
  }
}

pub(crate) const MODELS: &[Model] = &[
  // The whisper.cpp project's own conversion of OpenAI's Whisper base model,
  // at a fixed revision.
  Model {
    files: &[ModelFile {
      name: "ggml-base.bin",
      sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
      size_bytes: 147_951_465,
      url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.bin",
    }],
    id: "base",
    purpose: Purpose::Moments,
  },
  // Sidon v0.1 from the University of Tokyo (MIT), its released TorchScript
  // converted to ONNX: a speech encoder and the vocoder that speaks again
  // from what it hears.
  Model {
    files: &[
      ModelFile {
        name: "sidon-encoder.onnx",
        sha256: "df510e203dc644a68b85399e580e2e5a7c4069af7a235dc670dd617aab5a5f1a",
        size_bytes: 795_539_651,
        url: "https://github.com/barrett-snapshot/voice-isolation-models/releases/download/v1/sidon_fe.onnx",
      },
      ModelFile {
        name: "sidon-vocoder.onnx",
        sha256: "8daa3f39d52d27b6fc4fd99258595f7a445c1a1dd77862a800d11000635581e8",
        size_bytes: 209_854_938,
        url: "https://github.com/barrett-snapshot/voice-isolation-models/releases/download/v1/sidon_decoder.onnx",
      },
    ],
    id: "sidon-v0.1",
    purpose: Purpose::StudioSound,
  },
];

pub(crate) fn find(id: &str) -> Result<&'static Model, String> {
  MODELS
    .iter()
    .find(|model| model.id == id)
    .ok_or_else(|| format!("There is no model called {id}"))
}

/// The model chosen for `purpose`.
pub(crate) fn for_purpose(purpose: Purpose) -> Option<&'static Model> {
  MODELS.iter().find(|model| model.purpose == purpose)
}
