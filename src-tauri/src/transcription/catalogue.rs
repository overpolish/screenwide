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
  /// Studio ducking: the singing in the system audio moved out of the
  /// voice's way.
  StudioDucking,
  /// Background removal: the person in the camera cut out of the room.
  CameraBackground,
  /// Objects picked in the camera, followed through the recording, to keep
  /// with the person or take away with the room.
  CameraObjects,
}

impl Purpose {
  /// What Settings calls the model for this use.
  pub(crate) fn name(self) -> String {
    match self {
      Self::Moments => crate::i18n::t!("settings-transcription-moments"),
      Self::StudioSound => crate::i18n::t!("settings-transcription-studio-sound"),
      Self::StudioDucking => crate::i18n::t!("settings-transcription-studio-ducking"),
      Self::CameraBackground => crate::i18n::t!("settings-transcription-camera-background"),
      Self::CameraObjects => crate::i18n::t!("settings-transcription-camera-objects"),
    }
  }

  /// What downloading the model for this use gets you.
  pub(crate) fn description(self) -> String {
    match self {
      Self::Moments => crate::i18n::t!("settings-transcription-moments-description"),
      Self::StudioSound => crate::i18n::t!("settings-transcription-studio-sound-description"),
      Self::StudioDucking => crate::i18n::t!("settings-transcription-studio-ducking-description"),
      Self::CameraBackground => {
        crate::i18n::t!("settings-transcription-camera-background-description")
      }
      Self::CameraObjects => crate::i18n::t!("settings-transcription-camera-objects-description"),
    }
  }

  /// Whether the model turns speech into text, so one transcribing model can
  /// stand in for another.
  pub(crate) fn transcribes(self) -> bool {
    match self {
      Self::Moments => true,
      Self::StudioSound | Self::StudioDucking | Self::CameraBackground | Self::CameraObjects => {
        false
      }
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
  // HT Demucs v4 from Meta (MIT), the single model rather than the
  // fine-tuned four, exported to ONNX with its weights stored at half
  // precision, at a fixed revision. It separates music into drums, bass,
  // other instruments and vocals.
  Model {
    files: &[ModelFile {
      name: "htdemucs.onnx",
      sha256: "d05c269d0178d2a72ad484b10b11dd370193fc923201c3b27a99f848745db70a",
      size_bytes: 165_612_636,
      url: "https://huggingface.co/adowu/htdemucs-onnx/resolve/e6b688a9a9f977e2efaba79d0f18b9b0cfc606ea/htdemucs_fp16weights.onnx",
    }],
    id: "htdemucs-v4",
    purpose: Purpose::StudioDucking,
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
  // which follows the object from frame to frame. Cutie's graphs have no
  // download address yet, so this model only arrives copied in by hand.
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
        url: "https://github.com/overpolish/screenwide/releases/download/v0.1.0-alpha/cutie-small-encode.onnx",
      },
      ModelFile {
        name: "cutie-small-read.onnx",
        sha256: "4dddb39b9a4d6d8331dfcc03ba348a3758a60603e12335f57f37cf0c0498983d",
        size_bytes: 67_372_954,
        url: "https://github.com/overpolish/screenwide/releases/download/v0.1.0-alpha/cutie-small-read.onnx",
      },
      ModelFile {
        name: "cutie-small-memorise.onnx",
        sha256: "f21d34bc45bf154a41023b0034388c8aa8231b02e14abd212f3d40a4c9452b3c",
        size_bytes: 38_035_664,
        url: "https://github.com/overpolish/screenwide/releases/download/v0.1.0-alpha/cutie-small-memorise.onnx",
      },
    ],
    id: "sam2.1-tiny-cutie-small",
    purpose: Purpose::CameraObjects,
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
