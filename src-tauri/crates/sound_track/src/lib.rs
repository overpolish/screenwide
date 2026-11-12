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
/// effects switches), the files their sounds are in, and the recorded
/// stream of the voice they play under and make way for, if the recording
/// has one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoundTrack {
  pub clips: Vec<RecordingAudioClip>,
  pub muted: Vec<AudioClipKind>,
  pub files: Vec<(String, SoundFile)>,
  pub voice: Option<usize>,
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
      self.voice.is_some(),
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
