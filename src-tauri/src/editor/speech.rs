// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone's speech tools. Remove silences (`silences`) cuts the long
//! pauses from one listen through the microphone track for speech
//! (`analysis`), kept beside the project so it runs once per recording.
//! Reduce noise (`noise`) cleans the track with a speech enhancer, Vocal
//! cleanup (`voice`) polishes the voice, and `heard` picks which of their
//! files the track is heard from. Auto volume (`auto_volume`) levels whatever
//! is heard to a steady loudness.

mod analysis;
pub(crate) mod auto_volume;
mod choice_file;
pub(crate) mod commands;
pub(crate) mod heard;
mod noise;
mod pause_gate;
mod silences;
#[cfg(test)]
mod tests;
mod voice;

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::{AudioTrackKind, EditorArtifact, EditorState};

/// What the speech tools read of the recording open in the editor.
struct SpeechSource {
  duration_ms: u64,
  microphone: usize,
  movie: PathBuf,
  /// The folder the project keeps its files in, where the listen is kept.
  project_folder: PathBuf,
}

impl SpeechSource {
  /// The recording open in the editor, if it is `artifact_id` and has a
  /// microphone track to listen to.
  fn open(app: &AppHandle, artifact_id: u64) -> Result<Self, String> {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Recording {
      audio_tracks,
      duration_ms,
      id,
      path,
      project,
      ..
    }) = artifact.as_ref()
    else {
      return Err("There is no recording open".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer available in the editor".to_owned());
    }
    Ok(Self {
      duration_ms: *duration_ms,
      microphone: audio_tracks
        .iter()
        .find(|track| track.kind == AudioTrackKind::Microphone)
        .map(|track| track.stream_index)
        .ok_or_else(|| "This recording has no microphone".to_owned())?,
      movie: path.clone(),
      project_folder: project
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
    })
  }
}

/// The voice activity model, bundled as a resource, or beside the program
/// when it runs unbundled during development.
fn vad_model(app: &AppHandle) -> Result<PathBuf, String> {
  const NAME: &str = "silero-vad.bin";
  let bundled = app
    .path()
    .resource_dir()
    .ok()
    .map(|resources| resources.join("models").join(NAME));
  let beside = std::env::current_exe()
    .ok()
    .and_then(|program| program.parent().map(|folder| folder.join(NAME)));
  bundled
    .into_iter()
    .chain(beside)
    .find(|path| path.is_file())
    .ok_or_else(|| "The voice activity model is missing from this installation".to_owned())
}

/// `selection` with each of the `streams` it carries read as it is heard:
/// from the file Reduce noise or Vocal cleanup made of it, and through Auto
/// volume's filters.
pub(crate) fn as_heard(
  selection: super::track_selection::TrackSelection,
  project_folder: &Path,
  streams: &[usize],
) -> super::track_selection::TrackSelection {
  selection
    .with_cleaned(heard::heard_files(project_folder, streams))
    .with_filters(auto_volume::heard_filters(project_folder, streams))
}
