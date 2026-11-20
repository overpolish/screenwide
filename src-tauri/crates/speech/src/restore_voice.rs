// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Restore voice: the microphone restored by a speech restoration model,
//! run in the transcriber program, given a studio microphone's finish
//! (`master`), and made once into a file of its own in the project. Long
//! quiet waits are left out of the restoring and come out silent
//! (`waits`), and the pauses are turned down, closer to the speech than
//! Reduce noise turns them (`gate`). While it is on it is heard in place of
//! Reduce noise and Vocal cleanup; Auto volume levels it as it would any
//! other file. It needs a model downloaded first, from Settings or from the
//! switch.
//!
//! Restoring a long recording takes minutes, so what it has restored is
//! kept as it goes (`restoring_path`): work stopped part-way, by the switch
//! going off or the app closing, goes on from there the next time.

mod gate;
mod level;
mod master;
mod waits;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::{CLEAN_SAMPLE_RATE, SAMPLE_RATE};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::analysis;
use super::choice_file;
use crate::runner::samples::decode_audio_stream;
use crate::runner::Transcriber;

/// Bumped when a file of an older make should be made again.
const FORMAT_VERSION: u16 = 5;
/// How much of the progress bar listening for speech takes, when it has not
/// been listened for already, and finishing after; restoring is many times
/// slower than either.
const LISTEN_SHARE: f32 = 0.05;
const FINISH_SHARE: f32 = 0.05;

/// Whether the microphone is restored.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum RestoreVoice {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("restore-voice-{stream}.json"))
}

/// The `stream`th track restored, in the project.
pub(super) fn path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("restored-voice-{stream}.flac"))
}

/// What has been restored of the `stream`th track so far, before its finish:
/// raw samples at the cleaning rate, kept until the track's file is made.
fn restoring_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("restored-voice-{stream}.restoring.f32"))
}

/// What versions before Restore voice kept, as Studio sound.
fn studio_sound_paths(project_folder: &Path, stream: usize) -> [PathBuf; 2] {
  [
    project_folder.join(format!("studio-{stream}.json")),
    project_folder.join(format!("studio-{stream}.flac")),
  ]
}

/// What the project says about the `stream`th track. A recording starts
/// with Restore voice on, until someone turns it off; it is heard only once
/// its file is made, which needs the model.
pub fn choice(project_folder: &Path, stream: usize) -> RestoreVoice {
  choice_file::read(&choice_path(project_folder, stream), FORMAT_VERSION)
    .unwrap_or(RestoreVoice::On)
}

pub fn keep(project_folder: &Path, stream: usize, choice: RestoreVoice) -> Result<(), String> {
  forget_older(project_folder, stream)?;
  choice_file::write(&choice_path(project_folder, stream), FORMAT_VERSION, choice)
    .map_err(|error| format!("Could not keep the Restore voice choice: {error}"))
}

/// Removes files made by an older make, then marks the choice as kept by
/// this one, so a file is only ever there when it is current. What was
/// chosen for the older make goes with it, and the switch starts again from
/// on, as a new recording does.
fn forget_older(project_folder: &Path, stream: usize) -> Result<(), String> {
  let path = choice_path(project_folder, stream);
  if choice_file::read::<RestoreVoice>(&path, FORMAT_VERSION).is_some() {
    return Ok(());
  }
  let _ = std::fs::remove_file(self::path(project_folder, stream));
  let _ = std::fs::remove_file(restoring_path(project_folder, stream));
  for older in studio_sound_paths(project_folder, stream) {
    let _ = std::fs::remove_file(older);
  }
  choice_file::write(&path, FORMAT_VERSION, RestoreVoice::On)
    .map_err(|error| format!("Could not keep the Restore voice choice: {error}"))
}

/// Whether the `stream`th track has been restored, by the current make.
pub fn is_made(project_folder: &Path, stream: usize) -> bool {
  choice_file::read::<RestoreVoice>(&choice_path(project_folder, stream), FORMAT_VERSION).is_some()
    && path(project_folder, stream).is_file()
}

/// Whether the `stream`th track is heard restored: the switch is on and the
/// file made, whether or not the model is still on this computer.
pub(super) fn is_heard(project_folder: &Path, stream: usize) -> bool {
  choice(project_folder, stream) == RestoreVoice::On && is_made(project_folder, stream)
}

/// One restoring at a time, so a switch turned while a recording is readied
/// on opening waits for the file rather than writing it a second time.
static MAKING: Mutex<()> = Mutex::new(());

/// Restores the `stream`th track of `movie` into the project with the models
/// at `models`, unless a file of this make is there already, telling
/// `progress` how far it has got, and only once there is work to do. With
/// the voice activity model at `vad`, long quiet waits are left out and the
/// pauses are turned down. Work stopped part-way (`crate::stop`) keeps what
/// was restored for the next time.
pub fn prepare(
  movie: &Path,
  stream: usize,
  project_folder: &Path,
  (models, vad): ((PathBuf, PathBuf), Option<PathBuf>),
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let _making = MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  forget_older(project_folder, stream)?;
  if is_made(project_folder, stream) {
    return Ok(());
  }
  progress(0.0);
  let restoring = restoring_path(project_folder, stream);
  forget_restoring_of_another_recording(&restoring, movie);
  let recorded = decode_audio_stream(movie, stream, SAMPLE_RATE)?;
  let speech = vad
    .map(|vad| {
      analysis::speech_map(movie, stream, project_folder, vad, &mut |fraction| {
        progress(fraction * LISTEN_SHARE);
      })
    })
    .transpose()?;
  let stretches = match &speech {
    Some(speech) => waits::stretches(speech, recorded.path())?,
    None => vec![[0, u64::MAX]],
  };
  Transcriber::start()?.restore_speech(
    models,
    &recorded,
    stretches,
    &restoring,
    &mut |fraction| {
      progress(LISTEN_SHARE + fraction * (1.0 - LISTEN_SHARE - FINISH_SHARE));
    },
  )?;
  let levels = (
    level::voiced(&restoring, CLEAN_SAMPLE_RATE)?,
    level::voiced(recorded.path(), SAMPLE_RATE)?,
  );
  drop(recorded);
  let gated = gate::level_and_gate(&restoring, levels, speech.as_ref())?;
  let destination = path(project_folder, stream);
  // Written beside the destination and moved over it whole, so a finish cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  if let Err(error) = master::master(&gated, levels.1, &partial) {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the restored voice: {error}"))?;
  let _ = std::fs::remove_file(&restoring);
  progress(1.0);
  Ok(())
}

/// What was restored before belongs to the recording as it was then; a
/// recording changed since starts again.
fn forget_restoring_of_another_recording(restoring: &Path, movie: &Path) {
  let modified = |path: &Path| std::fs::metadata(path).and_then(|metadata| metadata.modified());
  if let (Ok(restored), Ok(recorded)) = (modified(restoring), modified(movie)) {
    if recorded > restored {
      let _ = std::fs::remove_file(restoring);
    }
  }
}
