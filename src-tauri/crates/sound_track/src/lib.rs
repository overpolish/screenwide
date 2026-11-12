// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's sound track: songs and sound effects laid on its timeline
//! as audio clips (`clip`), the library they come from (`store`), and how
//! they are heard: laid out as pieces of their files (`pieces`) and played
//! through an FFmpeg filter graph (`graph`) that the export mixes with the
//! recorded tracks and the preview plays beside them.

pub mod clip;
pub mod graph;
pub mod pieces;
pub mod store;

pub use clip::{validate_clips, AudioClipKind, RecordingAudioClip};
pub use pieces::{pieces, Piece, SoundFile};

/// What the export and the preview need to hear a timeline's clips: the
/// clips, the kinds of clip left out (the Tracks island's Music and Sound
/// effects switches), the files their sounds are in, the recorded stream of
/// the voice they play under, if the recording has one, how loud that voice
/// is heard, which the clips are levelled against, and whether the songs
/// make way for it: Studio ducking on, and the voice heard.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoundTrack {
  pub clips: Vec<RecordingAudioClip>,
  pub muted: Vec<AudioClipKind>,
  pub files: Vec<(String, SoundFile)>,
  pub voice: Option<usize>,
  /// In LUFS.
  pub voice_lufs: Option<f64>,
  pub ducks: bool,
}

impl SoundTrack {
  /// The pieces the clips heard play on a timeline `timeline_ms` long.
  pub fn pieces(&self, timeline_ms: u64) -> Vec<Piece> {
    let heard: Vec<RecordingAudioClip> = self
      .clips
      .iter()
      .filter(|clip| !self.muted.contains(&clip.kind))
      .cloned()
      .collect();
    pieces(
      &heard,
      |name| {
        self
          .files
          .iter()
          .find_map(|(each, file)| (each == name).then_some(file))
      },
      timeline_ms,
      (self.voice.and(self.voice_lufs), self.ducks),
    )
  }

  /// The clips' sounds found as `resolve` finds them, each once.
  pub fn with_files(mut self, resolve: impl Fn(&str) -> Option<SoundFile>) -> Self {
    let mut files: Vec<(String, SoundFile)> = Vec::new();
    for clip in &self.clips {
      if files.iter().any(|(name, _)| *name == clip.sound) {
        continue;
      }
      if let Some(file) = resolve(&clip.sound) {
        files.push((clip.sound.clone(), file));
      }
    }
    self.files = files;
    self
  }
}
