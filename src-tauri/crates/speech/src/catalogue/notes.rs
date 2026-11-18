// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The meeting notes download: the models the notes are taken with, and the
//! one that finds what may be private in them.

use super::{Model, ModelFile, Purpose, PARAKEET_ULTRA};

/// How many of [`NOTES`]' files the notes are taken with; the rest find what
/// may be private in them. Someone who downloaded the notes before those
/// were added keeps taking notes without them.
const SPEECH_FILES: usize = 3;

// Meeting notes: Parakeet Ultra writes each stretch of a call, and NVIDIA's
// Nemotron 3 Diarization (OpenMDW-1.1, its own 8-bit GGUF at a fixed
// revision) tells up to eight voices apart over the whole call, twelve
// minutes in under two seconds on an M4 Pro. A call with more is told apart
// in windows joined by each voice's fingerprint from 3D-Speaker's CAM++
// (English, VoxCeleb; Apache-2.0), in sherpa-onnx's ONNX export. What may be
// private in the notes is found by GLiNER-PII base (Knowledgator and
// Wordcab, Apache-2.0) in its authors' 8-bit ONNX export with its
// tokenizer, at a fixed revision: of its edge, small and base models it
// found the most in meeting-notes paragraphs with the fewest false finds.
pub(super) const NOTES: Model = Model {
  files: &[
    PARAKEET_ULTRA,
    ModelFile {
      name: "Nemotron-3-Diarization.q8_0.gguf",
      sha256: "08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1",
      size_bytes: 107_012_128,
      url: "https://huggingface.co/nvidia/Nemotron-3-Diarization/resolve/52a432672732425e4c19765b9fec9b5810e6f4ba/Nemotron-3-Diarization.q8_0.gguf",
    },
    ModelFile {
      name: "3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx",
      sha256: "357a834f702b80161e5b981182c038e18553c1f2ca752ed6cec2052365d4129b",
      size_bytes: 29_596_978,
      url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx",
    },
    ModelFile {
      name: "gliner-pii-base-v1.0-q8.onnx",
      sha256: "0514c8fd86d0513ce5351a3267f132b57d5bcd8f99a90d43cde1228092881d19",
      size_bytes: 196_757_174,
      url: "https://huggingface.co/knowledgator/gliner-pii-base-v1.0/resolve/61726e0ad791dcab3e29339bbec3ad42ded65641/onnx/model_quint8.onnx",
    },
    ModelFile {
      name: "gliner-pii-base-v1.0-tokenizer.json",
      sha256: "ee028763434d18611c1c36356ea1d050e90a9fa94ede57fac48b39f85f818ad1",
      size_bytes: 8_649_232,
      url: "https://huggingface.co/knowledgator/gliner-pii-base-v1.0/resolve/61726e0ad791dcab3e29339bbec3ad42ded65641/tokenizer.json",
    },
  ],
  id: "notes-ultra-nemotron-3-diarization-q8_0",
  purpose: Purpose::Notes,
};

/// The meeting notes model.
pub fn notes() -> &'static Model {
  &NOTES
}

/// The files meeting notes are taken with: the speech model, the speaker
/// model and the voice model, in that order.
pub fn notes_speech_files() -> &'static [ModelFile] {
  &NOTES.files[..SPEECH_FILES]
}

/// The model that finds what may be private in notes, and its tokenizer.
pub fn notes_sensitive_files() -> (&'static ModelFile, &'static ModelFile) {
  (&NOTES.files[SPEECH_FILES], &NOTES.files[SPEECH_FILES + 1])
}
