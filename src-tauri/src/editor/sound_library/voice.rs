// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The voice the songs and sound effects play under: the recording's
//! microphone, how loud it is heard, which the clips are levelled against,
//! and whether the songs make way for it.

use std::path::Path;

use screenwide_export::recording_model::AudioTrackKind;

/// The folder the project whose manifest is `project` keeps its files in.
fn folder(project: &Path) -> &Path {
  project.parent().unwrap_or(Path::new("."))
}

/// The recorded stream of the microphone of the project whose manifest is
/// `project`, if it has one.
pub(super) fn microphone(project: &Path) -> Option<usize> {
  screenwide_project::read(project)
    .ok()
    .and_then(|manifest| manifest.recording)
    .and_then(|recording| {
      super::super::artifact_present::recording_audio_tracks(
        recording.has_system_audio,
        recording.has_microphone,
      )
      .into_iter()
      .find(|track| track.kind == AudioTrackKind::Microphone)
    })
    .map(|track| track.stream_index)
}

/// How loud the `microphone`th track of the project whose manifest is
/// `project` is heard, in LUFS.
pub(super) fn voice_lufs(project: &Path, microphone: usize) -> f64 {
  screenwide_speech::auto_volume::voice_lufs(folder(project), microphone)
}

/// Whether the songs of the project whose manifest is `project` make way
/// for its `microphone`th track: Studio ducking is on for it and has its
/// model. Whether the microphone is heard is for the caller to say.
pub(crate) fn ducks(app: &tauri::AppHandle, project: &Path, microphone: usize) -> bool {
  crate::editor::speech::studio_ducking::wanted(app, folder(project), microphone).is_some()
}
