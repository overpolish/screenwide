// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captions written out as subtitle files a player or an editor reads
//! beside the video: SubRip and WebVTT.

use crate::captions::Caption;

/// A kind of subtitle file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubtitleFile {
  /// SubRip, which nearly every player and editor reads.
  Srt,
  /// WebVTT, which browsers read.
  Vtt,
}

impl SubtitleFile {
  pub fn extension(self) -> &'static str {
    match self {
      Self::Srt => "srt",
      Self::Vtt => "vtt",
    }
  }
}

/// The captions as a subtitle file of `kind`.
pub fn subtitles(captions: &[Caption], kind: SubtitleFile) -> String {
  match kind {
    SubtitleFile::Srt => srt(captions),
    SubtitleFile::Vtt => vtt(captions),
  }
}

/// `ms` as hours, minutes, seconds and milliseconds, the last after
/// `separator`: a comma in SubRip, a full stop in WebVTT.
fn timestamp(ms: f64, separator: char) -> String {
  let total = ms.max(0.0).round() as u64;
  let (hours, rest) = (total / 3_600_000, total % 3_600_000);
  let (minutes, rest) = (rest / 60_000, rest % 60_000);
  let (seconds, millis) = (rest / 1_000, rest % 1_000);
  format!("{hours:02}:{minutes:02}:{seconds:02}{separator}{millis:03}")
}

/// The captions as a SubRip (`.srt`) file.
pub fn srt(captions: &[Caption]) -> String {
  let mut file = String::new();
  for (index, caption) in captions.iter().enumerate() {
    file.push_str(&format!(
      "{}\n{} --> {}\n{}\n\n",
      index + 1,
      timestamp(caption.start_ms, ','),
      timestamp(caption.end_ms, ','),
      caption.text,
    ));
  }
  file
}

/// The captions as a WebVTT (`.vtt`) file. A caption's text is escaped, as
/// WebVTT reads `<` and `&` as markup.
pub fn vtt(captions: &[Caption]) -> String {
  let mut file = String::from("WEBVTT\n\n");
  for caption in captions {
    let text = caption
      .text
      .replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace('>', "&gt;");
    file.push_str(&format!(
      "{} --> {}\n{}\n\n",
      timestamp(caption.start_ms, '.'),
      timestamp(caption.end_ms, '.'),
      text,
    ));
  }
  file
}
