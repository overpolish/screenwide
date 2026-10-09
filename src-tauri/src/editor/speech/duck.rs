// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system audio making way for the voice, as part of Auto volume. While
//! the voice speaks (`gate`), all of the system audio comes down a little,
//! and the bands the voice is using at that moment come down further
//! (`plan`), worked out in the short-time spectrum of both tracks
//! (`framer`). Where the voice is quiet the system audio is as recorded.
//!
//! It depends only on the recording, the microphone as recorded and the
//! speech heard in it, so it is made once into a file of its own in the
//! project, which preview and export play in place of the system audio
//! while Auto volume is on for the microphone it was made against.

mod framer;
mod gate;
mod pipe;
mod plan;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::CLEAN_SAMPLE_RATE as RATE;
use serde::{Deserialize, Serialize};

use self::framer::{Framer, HOP, SIZE};
use self::plan::{Plan, VoiceBands};
use super::auto_volume::{self, AutoVolume};
use super::{analysis, choice_file};

/// Bumped when a file of an older make should be made again.
const FORMAT_VERSION: u16 = 1;
/// How much of the progress bar listening for speech, reading the voice and
/// making way in the system audio each take.
const SHARES: [f32; 3] = [0.2, 0.3, 0.5];

/// What a file was made against.
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Made {
  microphone: usize,
}

fn made_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("duck-{stream}.json"))
}

/// The `stream`th track making way for the voice, in the project.
pub(super) fn path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("duck-{stream}.flac"))
}

/// The microphone the `stream`th track was made to make way for, if a file
/// of this make is there.
fn made(project_folder: &Path, stream: usize) -> Option<usize> {
  choice_file::read::<Made>(&made_path(project_folder, stream), FORMAT_VERSION)
    .filter(|_| path(project_folder, stream).is_file())
    .map(|made| made.microphone)
}

pub(super) fn is_made(project_folder: &Path, stream: usize) -> bool {
  made(project_folder, stream).is_some()
}

/// Whether the `stream`th track has been made to make way for the
/// `microphone`th, by the current make.
pub(super) fn is_made_for(project_folder: &Path, stream: usize, microphone: usize) -> bool {
  made(project_folder, stream) == Some(microphone)
}

/// Whether the `stream`th track is heard making way: its file is made and
/// Auto volume is on for the microphone it makes way for, which is among
/// `heard`, the tracks that are on.
pub(super) fn is_heard(project_folder: &Path, stream: usize, heard: &[usize]) -> bool {
  made(project_folder, stream).is_some_and(|microphone| {
    heard.contains(&microphone) && auto_volume::choice(project_folder, microphone) == AutoVolume::On
  })
}

/// The tracks made to make way, with the microphone each makes way for.
pub(crate) fn ducked_by(project_folder: &Path, streams: &[usize]) -> Vec<(usize, usize)> {
  streams
    .iter()
    .filter_map(|&stream| made(project_folder, stream).map(|microphone| (stream, microphone)))
    .collect()
}

/// One file made at a time, so the editor opening a recording and an export
/// started meanwhile do not both write it.
static MAKING: Mutex<()> = Mutex::new(());

/// Makes the `system`th track of `movie` make way for the `microphone`th,
/// unless a file of this make is there already, telling `progress` how far
/// it has got, and only once there is work to do.
pub(super) fn prepare(
  movie: &Path,
  (system, microphone): (usize, usize),
  project_folder: &Path,
  model: PathBuf,
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
  let speech = analysis::speech_map(movie, microphone, project_folder, model, &mut |fraction| {
    progress(fraction * SHARES[0]);
  })?;
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
  let mut plan = Plan::new(voice, gate, RATE);
  let destination = path(project_folder, system);
  // Written beside the destination and moved over it whole, so work cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  let written = pipe::make_way(movie, system, &partial, &mut plan, &mut |read| {
    progress(SHARES[0] + SHARES[1] + SHARES[2] * (read as f64 / expected).min(1.0) as f32);
  });
  if let Err(error) = written {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the system audio: {error}"))?;
  choice_file::write(
    &made_path(project_folder, system),
    FORMAT_VERSION,
    Made { microphone },
  )
  .map_err(|error| format!("Could not keep the system audio: {error}"))
}
