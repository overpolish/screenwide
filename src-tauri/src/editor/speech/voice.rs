// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Vocal cleanup: the microphone track with its ticks taken down (`tick`),
//! its mouth clicks taken out (`declick`) and its plosives' thumps softened
//! (`deplosive`), the last two in the spectrum (`spectrum`), its harsh "s"
//! sounds softened (`deess`) and its rumble and boxiness eased (`biquad`),
//! at the loudness it had and with its peaks held below clipping (`encode`).
//! Evening out its level is Auto volume's work.
//! Like Reduce noise, it is made once into a file of its own in the project,
//! which preview and export play in its place while the switch is on. It is
//! made from what Reduce noise leaves, the track as recorded or the track
//! cleaned of noise, into a file for each, so either switch can turn without
//! the other's work being redone.

mod biquad;
mod declick;
mod deess;
mod deplosive;
mod encode;
mod level;
#[cfg(test)]
mod plosive_tests;
mod spectrum;
#[cfg(test)]
mod tests;
mod tick;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::CLEAN_SAMPLE_RATE;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{choice_file, noise};
use crate::transcription::runner::{decode_audio_stream, Samples};

/// Bumped when a cleaned file of an older make should be made again.
const FORMAT_VERSION: u16 = 8;

/// Whether the microphone's voice is cleaned up.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum VocalCleanup {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("voice-{stream}.json"))
}

/// The `stream`th track cleaned up, in the project: from the track as
/// recorded, or with `denoised` from the file Reduce noise made of it.
pub(super) fn cleaned_path(project_folder: &Path, stream: usize, denoised: bool) -> PathBuf {
  let name = if denoised { "noise-voice" } else { "voice" };
  project_folder.join(format!("{name}-{stream}.flac"))
}

/// What the project says about the `stream`th track's vocal cleanup. A
/// recording starts with Vocal cleanup on, until someone turns it off.
pub(super) fn choice(project_folder: &Path, stream: usize) -> VocalCleanup {
  choice_file::read(&choice_path(project_folder, stream), FORMAT_VERSION)
    .unwrap_or(VocalCleanup::On)
}

pub(super) fn keep(
  project_folder: &Path,
  stream: usize,
  choice: VocalCleanup,
) -> Result<(), String> {
  forget_older(project_folder, stream)?;
  choice_file::write(&choice_path(project_folder, stream), FORMAT_VERSION, choice)
    .map_err(|error| format!("Could not keep the vocal cleanup choice: {error}"))
}

/// Whether the `stream`th track has been cleaned up from the source
/// `denoised` names, by the current cleanup.
pub(super) fn is_made(project_folder: &Path, stream: usize, denoised: bool) -> bool {
  choice_file::read::<VocalCleanup>(&choice_path(project_folder, stream), FORMAT_VERSION).is_some()
    && cleaned_path(project_folder, stream, denoised).is_file()
}

/// Removes files made by an older cleanup, then marks the choice as kept by
/// this one, so a file is only ever there when it is current. What was
/// chosen for the older make goes with it, and the switch starts again from
/// on, as a new recording does.
fn forget_older(project_folder: &Path, stream: usize) -> Result<(), String> {
  let path = choice_path(project_folder, stream);
  if choice_file::read::<VocalCleanup>(&path, FORMAT_VERSION).is_some() {
    return Ok(());
  }
  for denoised in [false, true] {
    let _ = std::fs::remove_file(cleaned_path(project_folder, stream, denoised));
  }
  choice_file::write(&path, FORMAT_VERSION, VocalCleanup::On)
    .map_err(|error| format!("Could not keep the vocal cleanup choice: {error}"))
}

/// One cleanup at a time, so a switch turned while a recording is readied on
/// opening waits for the file rather than writing it a second time.
static MAKING: Mutex<()> = Mutex::new(());

/// Cleans up the `stream`th track of `movie` into the project, from the
/// track as recorded or, with `denoised`, from the file Reduce noise made of
/// it, unless a file of this make is there already. Tells `progress` how
/// far it has got, and only once there is work to do.
pub(super) fn prepare(
  movie: &Path,
  stream: usize,
  project_folder: &Path,
  denoised: bool,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let _making = MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  forget_older(project_folder, stream)?;
  let destination = cleaned_path(project_folder, stream, denoised);
  if destination.is_file() {
    return Ok(());
  }
  progress(0.0);
  let samples = if denoised {
    decode_audio_stream(
      &noise::cleaned_path(project_folder, stream),
      0,
      CLEAN_SAMPLE_RATE,
    )?
  } else {
    decode_audio_stream(movie, stream, CLEAN_SAMPLE_RATE)?
  };
  // Written beside the destination and moved over it whole, so a cleanup cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  if let Err(error) = encode::encode(&samples, &partial, true, progress) {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the cleaned up microphone: {error}"))
}

/// Cleans up the voice Studio sound rebuilt, the samples in `rebuilt` at the
/// cleaning rate, into a FLAC file at `output`, telling `progress` how far it
/// has got. Every stage runs but the plosives': it takes the bursts of a
/// laugh for plosives, and softening them is heard on the rebuilt voice.
pub(super) fn clean_up_rebuilt(
  rebuilt: &Samples,
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  encode::encode(rebuilt, output, false, progress)
}
