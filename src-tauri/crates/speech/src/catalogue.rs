// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod dictation;
mod notes;
mod retired;

pub use dictation::{dictation, dictation_cleanup, dictation_description, Writes};
pub use notes::{notes, notes_sensitive_files, notes_speech_files};
pub use retired::{RETIRED_FILES, RETIRED_FOLDER_PREFIXES};
use serde::Serialize;

/// What a model is for. Each use has its own model, chosen for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Purpose {
  /// Restore voice: the microphone restored and given a studio
  /// microphone's finish.
  RestoreVoice,
  /// Smart ducking: the singing in the system audio moved out of the
  /// voice's way.
  SmartDucking,
  /// Background removal: the person in the camera cut out of the room.
  CameraBackground,
  /// Objects picked in the camera, followed through the recording, to keep
  /// with the person or take away with the room.
  CameraObjects,
  /// Speech written out as text: the recording's microphone word by word, to
  /// edit and caption by, and moments' voice notes. Both take the accurate
  /// model, so a note reads as well as the transcript.
  Transcripts,
  /// Dictation: speech typed into any app. English takes a model that
  /// streams, the words appearing as they are spoken; other languages one
  /// that writes each stretch of speech once a pause ends it
  /// ([`dictation`]).
  Dictation,
  /// Dictation's corrections: what the speaker corrected as they spoke
  /// ("at 2, actually 3") put right, in English.
  DictationCleanup,
  /// Meeting notes: a call's speech written out, the other speakers told
  /// apart.
  Notes,
}

impl Purpose {
  /// What Settings calls the model for this use.
  pub fn name(self) -> String {
    match self {
      Self::RestoreVoice => screenwide_i18n::t!("settings-transcription-restore-voice"),
      Self::SmartDucking => screenwide_i18n::t!("settings-transcription-smart-ducking"),
      Self::CameraBackground => screenwide_i18n::t!("settings-transcription-camera-background"),
      Self::CameraObjects => screenwide_i18n::t!("settings-transcription-camera-objects"),
      Self::Transcripts => screenwide_i18n::t!("settings-transcription-transcripts"),
      Self::Dictation => screenwide_i18n::t!("settings-transcription-dictation"),
      Self::DictationCleanup => screenwide_i18n::t!("settings-transcription-dictation-cleanup"),
      Self::Notes => screenwide_i18n::t!("settings-transcription-notes"),
    }
  }

  /// What downloading the model for this use gets you.
  pub fn description(self) -> String {
    match self {
      Self::RestoreVoice => {
        screenwide_i18n::t!("settings-transcription-restore-voice-description")
      }
      Self::SmartDucking => {
        screenwide_i18n::t!("settings-transcription-smart-ducking-description")
      }
      Self::CameraBackground => {
        screenwide_i18n::t!("settings-transcription-camera-background-description")
      }
      Self::CameraObjects => {
        screenwide_i18n::t!("settings-transcription-camera-objects-description")
      }
      Self::Transcripts => screenwide_i18n::t!("settings-transcription-transcripts-description"),
      Self::Dictation => screenwide_i18n::t!("settings-transcription-dictation-description"),
      Self::DictationCleanup => {
        screenwide_i18n::t!("settings-transcription-dictation-cleanup-description")
      }
      Self::Notes => screenwide_i18n::t!("settings-transcription-notes-description"),
    }
  }
}

/// One file of a model, fetched from a fixed address: a file that changed
/// there would fail its checksum rather than arrive unnoticed.
#[derive(Debug)]
pub struct ModelFile {
  pub name: &'static str,
  pub url: &'static str,
  pub size_bytes: u64,
  pub sha256: &'static str,
}

#[derive(Debug)]
pub struct Model {
  /// The model's own name, as transcripts record it.
  pub id: &'static str,
  /// What it is for, which also names it in Settings.
  pub purpose: Purpose,
  pub files: &'static [ModelFile],
}

impl Model {
  pub fn size_bytes(&self) -> u64 {
    self.files.iter().map(|file| file.size_bytes).sum()
  }
}

/// Parakeet Ultra from Moondream (CC-BY-4.0), NVIDIA's Parakeet TDT 0.6B v3
/// trained further, as the transcribe.cpp project converted it to GGUF at 8
/// bits, at a fixed revision: 25 European languages, written at each pause.
const PARAKEET_ULTRA: ModelFile = ModelFile {
  name: "parakeet-ultra-Q8_0.gguf",
  sha256: "007a59761e9258779f189df396b50d12b83bb5b79e66df4b955c230d2f2a0a59",
  size_bytes: 740_363_168,
  url: "https://huggingface.co/handy-computer/parakeet-ultra-gguf/resolve/a1539be1b849ea7463b88343e5277b373d0c4ea9/parakeet-ultra-Q8_0.gguf",
};

pub const MODELS: &[Model] = &[
  // Whisper large-v3-turbo, quantised to 8 bits by the whisper.cpp project,
  // at a fixed revision: accurate enough to edit by, and quick for its size.
  Model {
    files: &[ModelFile {
      name: "ggml-large-v3-turbo-q8_0.bin",
      sha256: "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1",
      size_bytes: 874_188_075,
      url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo-q8_0.bin",
    }],
    id: "large-v3-turbo-q8_0",
    purpose: Purpose::Transcripts,
  },
  // Parakeet Unified EN 0.6B from NVIDIA (NVIDIA Open Model License), as the
  // transcribe.cpp project converted it to GGUF at 8 bits, at a fixed
  // revision: English that streams, punctuated as it is said. With it,
  // Parakeet Ultra writes each stretch again once a pause ends it, hearing
  // the words after each word as the stream cannot: half the mistakes on the
  // dictations in `crates/dictation/evaluation`, about 40 ms a stretch.
  Model {
    files: &[
      ModelFile {
        name: "parakeet-unified-en-0.6b-Q8_0.gguf",
        sha256: "4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795",
        size_bytes: 731_357_568,
        url: "https://huggingface.co/handy-computer/parakeet-unified-en-0.6b-gguf/resolve/843b1b175ace04f5a89d379c79e2518c0d065b25/parakeet-unified-en-0.6b-Q8_0.gguf",
      },
      PARAKEET_ULTRA,
    ],
    id: "parakeet-unified-en-0.6b-q8_0",
    purpose: Purpose::Dictation,
  },
  Model {
    files: &[PARAKEET_ULTRA],
    id: "parakeet-ultra-q8_0",
    purpose: Purpose::Dictation,
  },
  // SpeakoFlow Mini 0.8B from SpeakoFlow (Apache-2.0), Qwen3.5 0.8B trained
  // to clean up dictation, in its authors' GGUF at 8 bits, at a fixed
  // revision: it applies the corrections a speaker makes as they speak and
  // leaves clean text alone. It scored best of the models tried on the
  // dictations in `crates/dictation/evaluation`, and at 8 bits better than
  // at 4.
  Model {
    files: &[ModelFile {
      name: "SpeakoFlow-Mini-0.8B-Q8_0.gguf",
      sha256: "696769bb6911f51bc231b112926e934cf7bfc760e6cdfa24212907bc5ad41fc9",
      size_bytes: 833_591_776,
      url: "https://huggingface.co/SpeakoFlow/speakoflow-mini/resolve/835431771f72820251fe6c6b4b07f12b000e2647/SpeakoFlow-Mini-0.8B-Q8_0.gguf",
    }],
    id: "speakoflow-mini-0.8b-q8_0",
    purpose: Purpose::DictationCleanup,
  },
  notes::NOTES,
  // UniPASE from Nanjing University (Apache-2.0), exported to ONNX as two
  // models: one restores the voice at 16 kHz, its weights stored at half
  // precision and its graph written so DirectML runs it; the other adds the
  // band above 8 kHz. Each keeps its weights in a file beside it, under the
  // name the model looks for.
  Model {
    files: &[
      ModelFile {
        name: "unipase-restore.onnx",
        sha256: "c3d9e66b85b891b7659536c7b3fbfd2fcc0394723fb7498ca3dd02eb2c2a9b3b",
        size_bytes: 1_476_912,
        url: "https://huggingface.co/overpolish/unipase-onnx/resolve/ab6bf6fe847846d5f096e6c7da9768086168b891/unipase-restore.onnx",
      },
      ModelFile {
        name: "unipase-restore.onnx.data",
        sha256: "b689f4bf1e27f8cae473559e945e3f15832b344e3409da7129c6ae749e17a580",
        size_bytes: 1_091_734_536,
        url: "https://huggingface.co/overpolish/unipase-onnx/resolve/ab6bf6fe847846d5f096e6c7da9768086168b891/unipase-restore.onnx.data",
      },
      ModelFile {
        name: "unipase-widen.onnx",
        sha256: "b2efe957b414569cceb82a8528783e57be7d014b501aa2f9c7967606d90a6fcc",
        size_bytes: 376_606,
        url: "https://huggingface.co/overpolish/unipase-onnx/resolve/ab6bf6fe847846d5f096e6c7da9768086168b891/unipase-widen.onnx",
      },
      ModelFile {
        name: "unipase-widen.onnx.data",
        sha256: "3b3ebe46adc8c2536d12cb48beaf58e13adef16fb1718b8a09c6be2e4cbaff11",
        size_bytes: 29_980_420,
        url: "https://huggingface.co/overpolish/unipase-onnx/resolve/ab6bf6fe847846d5f096e6c7da9768086168b891/unipase-widen.onnx.data",
      },
    ],
    id: "unipase",
    purpose: Purpose::RestoreVoice,
  },
  // Kimberley Jensen's Mel-Band RoFormer vocals model (MIT), as audiojs
  // exported it to ONNX for the GPU with its weights stored at half
  // precision, at a fixed revision. It takes the singing out of music.
  Model {
    files: &[ModelFile {
      name: "mel-roformer-vocals.onnx",
      sha256: "adcbbb826f12453fbb86652b503242d565a9c81cdef1bca2fe0e54752eba91ef",
      size_bytes: 464_050_097,
      url: "https://huggingface.co/audiojs/mel-roformer/resolve/c1aba02c00a0bee852d6754fe6ede8164f90a4b3/mel-roformer.fp16.onnx",
    }],
    id: "mel-roformer-vocals",
    purpose: Purpose::SmartDucking,
  },
  // Robust Video Matting from ByteDance (GPL-3.0), its MobileNetV3 model as
  // the authors released it in ONNX at full precision. It finds a person in
  // video, carrying what it saw from frame to frame.
  Model {
    files: &[ModelFile {
      name: "rvm-mobilenetv3.onnx",
      sha256: "88d4531297118f595bf2fd60f6f566aec2e559393802d1f436c380f0cbbd2828",
      size_bytes: 14_975_696,
      url: "https://github.com/PeterL1n/RobustVideoMatting/releases/download/v1.0.0/rvm_mobilenetv3_fp32.onnx",
    }],
    id: "rvm-mobilenetv3",
    purpose: Purpose::CameraBackground,
  },
  // SAM 2.1 Hiera-Tiny from Meta (Apache-2.0) at a fixed revision, its image
  // encoder and mask decoder, which turn a click into an object's mask; and
  // Cutie-small (Cheng et al., MIT), its weights trained without the
  // non-commercial MOSE set, exported to ONNX in three graphs of fixed size,
  // which follows the object from frame to frame. The read graph reads all of
  // its memory rather than Cutie's top 30 matches for each pixel: DirectML's
  // TopK took two thirds of every read, and the masks barely differ.
  Model {
    files: &[
      ModelFile {
        name: "sam2.1-tiny-vision-encoder.onnx",
        sha256: "aa7a8542942f042e235a993a1ab0ccf5f049918500577802a7f10ec1b39bb873",
        size_bytes: 134_335_567,
        url: "https://huggingface.co/square-zero-labs/sam2.1-tiny-video-onnx/resolve/3b2984dd865f6e9d2cc6aed0be6a5a5c2eb352ce/onnx/vision_encoder.onnx",
      },
      ModelFile {
        name: "sam2.1-tiny-mask-decoder.onnx",
        sha256: "0461896de3db00936fe1643506f129d71cb6d5d2ae15754811756b7ea1b070c6",
        size_bytes: 17_794_355,
        url: "https://huggingface.co/square-zero-labs/sam2.1-tiny-video-onnx/resolve/3b2984dd865f6e9d2cc6aed0be6a5a5c2eb352ce/onnx/mask_decoder.onnx",
      },
      ModelFile {
        name: "cutie-small-encode.onnx",
        sha256: "b192995b18f5c8b2c6ce3f80ed2cced1e03db0af423111e53ed51db9c0f538de",
        size_bytes: 12_851_335,
        url: "https://github.com/overpolish/screenwide/releases/download/models-1/cutie-small-encode.onnx",
      },
      ModelFile {
        name: "cutie-small-read.onnx",
        sha256: "f2bab53fa0e7451cab0f2b0c4893d82a671e0d6977340b5074323f76ca7127b3",
        size_bytes: 67_371_722,
        url: "https://github.com/overpolish/screenwide/releases/download/models-1/cutie-small-read.onnx",
      },
      ModelFile {
        name: "cutie-small-memorise.onnx",
        sha256: "f21d34bc45bf154a41023b0034388c8aa8231b02e14abd212f3d40a4c9452b3c",
        size_bytes: 38_035_664,
        url: "https://github.com/overpolish/screenwide/releases/download/models-1/cutie-small-memorise.onnx",
      },
    ],
    id: "sam2.1-tiny-cutie-small",
    purpose: Purpose::CameraObjects,
  },
];

pub fn find(id: &str) -> Result<&'static Model, String> {
  MODELS
    .iter()
    .find(|model| model.id == id)
    .ok_or_else(|| format!("There is no model called {id}"))
}

/// The model chosen for `purpose`; for dictation, English's ([`dictation`]
/// chooses by language).
pub fn for_purpose(purpose: Purpose) -> Option<&'static Model> {
  MODELS.iter().find(|model| model.purpose == purpose)
}
