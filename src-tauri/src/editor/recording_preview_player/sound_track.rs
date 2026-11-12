// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The songs and sound effects as the preview plays them. The editor sends
//! them with every play, with where on the timeline playback starts, and
//! they are laid out then: an edit made while paused is heard from the next
//! play on, as the export would hear it.

use std::path::Path;

use serde::Deserialize;

use screenwide_export::recording_model::{AudioTrackKind, RecordingAudioTrack};
use screenwide_sound_track::{graph, AudioClipKind, Piece, RecordingAudioClip};

/// The clips as the editor sends them with a play.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSoundTrackRequest {
  clips: Vec<RecordingAudioClip>,
  /// The kinds of clip left out.
  #[serde(default)]
  muted: Vec<AudioClipKind>,
  /// How long the edited timeline plays, in ms.
  timeline_ms: u64,
  /// Where on the timeline playback starts, in ms.
  start_ms: u64,
}

/// The clips laid out for one playback.
#[derive(Clone, Debug, Default)]
pub(super) struct PreviewSoundTrack {
  pieces: Vec<Piece>,
  start_ms: u64,
  timeline_ms: u64,
  /// The recorded stream of the voice the clips play under.
  voice: Option<usize>,
}

impl PreviewSoundTrack {
  /// `request`'s clips with their files found in the project whose manifest
  /// is `project`, at the levels they play at over `tracks`, making way for
  /// its microphone.
  pub(super) fn new(
    request: PreviewSoundTrackRequest,
    project: &Path,
    tracks: &[RecordingAudioTrack],
  ) -> Self {
    let voice = tracks
      .iter()
      .find(|track| track.kind == AudioTrackKind::Microphone)
      .map(|track| track.stream_index);
    let sound_track =
      crate::editor::sound_library::sound_track(project, (request.clips, request.muted), voice);
    Self {
      pieces: sound_track.pieces(request.timeline_ms),
      start_ms: request.start_ms.min(request.timeline_ms),
      timeline_ms: request.timeline_ms,
      voice,
    }
  }

  /// `pieces` as played from `start_ms` of a timeline `timeline_ms` long,
  /// making way for the recorded stream `voice`.
  #[cfg(test)]
  pub(super) fn laid_out(
    pieces: Vec<Piece>,
    (start_ms, timeline_ms): (u64, u64),
    voice: Option<usize>,
  ) -> Self {
    Self {
      pieces,
      start_ms,
      timeline_ms,
      voice,
    }
  }

  /// Whether anything is heard from where playback starts on.
  pub(super) fn is_heard(&self) -> bool {
    self
      .pieces
      .iter()
      .any(|piece| piece.end_ms > self.start_ms && piece.start_ms < self.timeline_ms)
  }

  /// The recorded stream the clips that make way listen to, where any do.
  pub(super) fn voice(&self) -> Option<usize> {
    self.voice.filter(|_| graph::makes_way(&self.pieces))
  }

  /// The clips mixed into `[soundmix]`, from where playback starts, or none
  /// where nothing is heard from there on, making way for `voice`, the
  /// voice's label in the graph, when given.
  pub(super) fn filter(&self, voice: Option<&str>) -> Option<String> {
    graph::filter(
      &self.pieces,
      (self.start_ms, self.timeline_ms),
      "soundmix",
      "",
      voice,
    )
  }
}
