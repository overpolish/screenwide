// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system audio making way for the voice, with Studio ducking on. While
//! the voice speaks (`gate`), all of the system audio comes down a little,
//! and the bands the voice is using at that moment come down further
//! (`plan`), worked out in the short-time spectrum of both tracks; the
//! singing in it is separated first (`singing`) and gets out of the way
//! entirely. Where the voice is quiet the system audio is as recorded.
//!
//! It depends only on the recording, the microphone as recorded and the
//! speech heard in it, so it is made once into a file of its own in the
//! project, which preview and export play in place of the system audio
//! while Studio ducking is on for the microphone it was made against, and
//! that microphone is heard.

mod choice;
mod gate;
mod pipe;
mod plan;
mod singing;
#[cfg(test)]
mod tests;

pub use choice::{choice, keep, StudioDucking};

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::CLEAN_SAMPLE_RATE as RATE;
use serde::{Deserialize, Serialize};

use self::plan::{Plan, VoiceBands};
use super::framer::{Framer, HOP, SIZE};
use super::{analysis, choice_file};

/// Bumped when a file of an older make should be made again.
const FORMAT_VERSION: u16 = 1;
/// How much of the progress bar listening for speech, reading the voice,
/// separating the singing and making way in the system audio each take.
const SHARES: [f32; 4] = [0.1, 0.1, 0.6, 0.2];

/// What a file was made against. Files made before Studio ducking say
/// nothing of singing: they had none separated, and are made again.
#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Made {
  microphone: usize,
  #[serde(default)]
  singing: bool,
}

fn made_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("duck-{stream}.json"))
}

/// The `stream`th track making way for the voice, in the project.
pub(super) fn path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("duck-{stream}.flac"))
}

/// What the `stream`th track was made against, if a file of this make is
/// there.
fn made(project_folder: &Path, stream: usize) -> Option<Made> {
  choice_file::read::<Made>(&made_path(project_folder, stream), FORMAT_VERSION)
    .filter(|_| path(project_folder, stream).is_file())
}

pub(super) fn is_made(project_folder: &Path, stream: usize) -> bool {
  made(project_folder, stream).is_some_and(|made| made.singing)
}

/// Whether the `stream`th track has been made to make way for the
/// `microphone`th, by the current make, with its singing separated.
pub fn is_made_for(project_folder: &Path, stream: usize, microphone: usize) -> bool {
  made(project_folder, stream)
    == Some(Made {
      microphone,
      singing: true,
    })
}

/// Whether the `stream`th track is heard making way: its file is made and
/// Studio ducking is on for the microphone it makes way for, which is among
/// `heard`, the tracks that are on.
pub(super) fn is_heard(project_folder: &Path, stream: usize, heard: &[usize]) -> bool {
  made(project_folder, stream).is_some_and(
    |Made {
       microphone,
       singing,
     }| {
      singing
        && heard.contains(&microphone)
        && choice(project_folder, microphone) == StudioDucking::On
    },
  )
}

/// The tracks made to make way, with the microphone each makes way for.
pub fn ducked_by(project_folder: &Path, streams: &[usize]) -> Vec<(usize, usize)> {
  streams
    .iter()
    .filter_map(|&stream| made(project_folder, stream).map(|made| (stream, made.microphone)))
    .collect()
}

/// Separates frames `from` to `to` of `sound`, a song from the sound
/// library, at the separation rate, with Studio ducking's model at `model`,
/// into the song's two parts, its singing and the rest of it: FLACs lined
/// up with the song's own timestamps, kept as they were elsewhere, at
/// `level` of their loudness. Tells `progress` how far it has got. Each
/// file is written beside its destination and moved over it whole.
pub fn separate_singing(
  sound: &Path,
  model: PathBuf,
  frames: (u64, u64),
  parts: (&Path, &Path, f32),
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  singing::into_song(sound, model, frames, parts, progress)
}

/// One file made at a time, so the editor opening a recording and an export
/// started meanwhile do not both write it.
static MAKING: Mutex<()> = Mutex::new(());

/// Makes the `system`th track of `movie` make way for the `microphone`th,
/// unless a file of this make is there already, with the voice activity
/// model and Studio ducking's separation model at `models`, telling
/// `progress` how far it has got, and only once there is work to do. The
/// singing in the track is separated and gets out of the way entirely.
pub fn prepare(
  movie: &Path,
  (system, microphone): (usize, usize),
  project_folder: &Path,
  (voice_activity, separation): (PathBuf, PathBuf),
  duration_ms: u64,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let _making = MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if is_made_for(project_folder, system, microphone) {
    return Ok(());
  }
  progress(0.0);
  let speech = analysis::speech_map(
    movie,
    microphone,
    project_folder,
    voice_activity,
    &mut |fraction| progress(fraction * SHARES[0]),
  )?;
  let expected = (duration_ms as f64 * f64::from(RATE) / 1_000.0).max(1.0);
  let mut voice = VoiceBands::default();
  let mut framer = Framer::new();
  let mut read = 0usize;
  pipe::decode(movie, microphone, 1, &mut |samples| {
    framer.feed(samples, None, &mut |_, spectrum| voice.add(spectrum, RATE));
    read += samples.len();
    progress(SHARES[0] + SHARES[1] * (read as f64 / expected).min(1.0) as f32);
  })?;
  let frames = ((expected as usize + SIZE) / HOP).max(1);
  let frame_ms = |frame: usize| Framer::middle(frame) * 1_000.0 / f64::from(RATE);
  let step_ms = HOP as f64 * 1_000.0 / f64::from(RATE);
  let gate = gate::gate(&speech, duration_ms as f64, frames, frame_ms, step_ms);
  let before_singing = SHARES[0] + SHARES[1];
  let singing = singing::separate(movie, system, separation, &mut |fraction| {
    progress(before_singing + SHARES[2] * fraction);
  })?;
  let singer = Plan::singer(&gate, RATE);
  let mut plans = vec![Plan::new(voice, gate, RATE), singer];
  let destination = path(project_folder, system);
  // Written beside the destination and moved over it whole, so work cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  let before_making = before_singing + SHARES[2];
  let written = pipe::make_way(
    movie,
    system,
    Some(singing.path()),
    &partial,
    &mut plans,
    &mut |read| {
      progress(before_making + SHARES[3] * (read as f64 / expected).min(1.0) as f32);
    },
  );
  if let Err(error) = written {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the system audio: {error}"))?;
  choice_file::write(
    &made_path(project_folder, system),
    FORMAT_VERSION,
    Made {
      microphone,
      singing: true,
    },
  )
  .map_err(|error| format!("Could not keep the system audio: {error}"))
}
