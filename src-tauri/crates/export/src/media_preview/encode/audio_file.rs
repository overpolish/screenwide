// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The file a recording's sound alone is saved as.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFile {
  /// AAC in an MP4 container, every track kept unless they are mixed.
  M4a,
  /// PCM in a WAV, which holds one stream, so the tracks are always mixed.
  Wav,
}

impl AudioFile {
  pub const fn extension(self) -> &'static str {
    match self {
      Self::M4a => "m4a",
      Self::Wav => "wav",
    }
  }

  pub(crate) const fn encoding(self) -> AudioEncoding {
    match self {
      Self::M4a => AudioEncoding::Aac,
      Self::Wav => AudioEncoding::Pcm,
    }
  }

  /// The muxer arguments. A WAV past 4 GB, about six hours of 16-bit
  /// stereo, is written as RF64, which a plain WAV header cannot size.
  pub(crate) const fn output_args(self) -> &'static [&'static str] {
    match self {
      Self::M4a => &EXPORT_MP4_OUTPUT,
      Self::Wav => &["-f", "wav", "-rf64", "auto"],
    }
  }

  /// The layout the file can hold: a WAV's one stream takes every track
  /// mixed.
  pub const fn layout(self, asked: AudioLayout) -> AudioLayout {
    match self {
      Self::M4a => asked,
      Self::Wav => AudioLayout::Mixdown,
    }
  }
}
