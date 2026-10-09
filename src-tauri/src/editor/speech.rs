// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone's speech tools. Remove silences (`silences`) cuts the long
//! pauses from one listen through the microphone track for speech
//! (`analysis`), kept beside the project so it runs once per recording.
//! Reduce noise (`noise`) cleans the track with a speech enhancer, Vocal
//! cleanup (`voice`) polishes the voice, and `heard` picks which of their
//! files the track is heard from. Auto volume (`auto_volume`) levels whatever
//! is heard to a steady loudness, and makes the system audio make way for
//! the voice while it speaks (`duck`).

mod analysis;
pub(crate) mod auto_volume;
mod choice_file;
pub(crate) mod commands;
pub(crate) mod duck;
mod framer;
pub(crate) mod heard;
mod noise;
mod pause_gate;
mod silences;
#[cfg(test)]
mod tests;
mod voice;

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::{AudioTrackKind, EditorArtifact, EditorState, RecordingAudioTrack};

/// What the speech tools read of the recording open in the editor.
struct SpeechSource {
  duration_ms: u64,
  microphone: usize,
  /// The system audio, which Auto volume makes make way for the voice.
  system: Option<usize>,
  /// Every audio track of the recording.
  streams: Vec<usize>,
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
      microphone: first_of(audio_tracks, AudioTrackKind::Microphone)
        .ok_or_else(|| "This recording has no microphone".to_owned())?,
      system: first_of(audio_tracks, AudioTrackKind::SystemAudio),
      streams: audio_tracks
        .iter()
        .map(|track| track.stream_index)
        .collect(),
      movie: path.clone(),
      project_folder: project
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
    })
  }
}

/// The stream of the first of `tracks` of `kind`.
fn first_of(tracks: &[RecordingAudioTrack], kind: AudioTrackKind) -> Option<usize> {
  tracks
    .iter()
    .find(|track| track.kind == kind)
    .map(|track| track.stream_index)
}

/// How much of Auto volume's bar measuring the voice takes, when the system
/// audio is made to make way for it after.
const MEASURE_SHARE: f32 = 0.4;

/// Readies Auto volume for the `microphone`th track of `movie`, where it is
/// on: measures the voice as it is heard, and, with `system` and the voice
/// activity model at `model`, makes the system audio make way for it.
fn ready_auto_volume(
  project_folder: &Path,
  (microphone, system): (usize, Option<usize>),
  (movie, duration_ms): (&Path, u64),
  model: Option<PathBuf>,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  if auto_volume::choice(project_folder, microphone) != auto_volume::AutoVolume::On {
    return Ok(());
  }
  let ducking = system.zip(model);
  let share = if ducking.is_some() {
    MEASURE_SHARE
  } else {
    1.0
  };
  auto_volume::measure(
    project_folder,
    microphone,
    movie,
    duration_ms,
    &mut |fraction| {
      progress(fraction * share);
    },
  )?;
  if let Some((system, model)) = ducking {
    duck::prepare(
      movie,
      (system, microphone),
      project_folder,
      model,
      duration_ms,
      &mut |fraction| progress(share + fraction * (1.0 - share)),
    )?;
  }
  Ok(())
}

/// Readies the microphone's switches for an export of the tracks `on` of a
/// recording, where they are on: the noise taken out, the voice cleaned up,
/// then Auto volume's measure and the system audio making way. An export
/// started while the editor is still readying them comes out as the preview
/// will sound; it waits for work already under way rather than doing it
/// again.
pub(crate) fn ready_for_export(
  app: &AppHandle,
  project_folder: &Path,
  tracks: &[RecordingAudioTrack],
  on: &[usize],
  recording: (&Path, u64),
) -> Result<(), String> {
  let on: Vec<RecordingAudioTrack> = tracks
    .iter()
    .filter(|track| on.contains(&track.stream_index))
    .cloned()
    .collect();
  let Some(microphone) = first_of(&on, AudioTrackKind::Microphone) else {
    return Ok(());
  };
  let model = vad_model(app).ok();
  let noise_on = noise::choice(project_folder, microphone) == noise::NoiseReduction::On;
  if let Some(model) = model.clone().filter(|_| noise_on) {
    noise::prepare(recording.0, microphone, project_folder, model, &mut |_| {})?;
  }
  if voice::choice(project_folder, microphone) == voice::VocalCleanup::On {
    let denoised = noise_on && noise::is_made(project_folder, microphone);
    voice::prepare(
      recording.0,
      microphone,
      project_folder,
      denoised,
      &mut |_| {},
    )?;
  }
  let system = first_of(&on, AudioTrackKind::SystemAudio);
  ready_auto_volume(
    project_folder,
    (microphone, system),
    recording,
    model,
    &mut |_| {},
  )
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
