// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reduce noise: the microphone track cleaned once into a file of its own in
//! the project, which preview and export play in its place while the switch
//! is on. The cleaning is DeepFilterNet's, run in the transcriber program: a
//! speech enhancer that keeps the voice and takes out what is not, steady
//! hiss and passing clicks alike. What it lets through between words is
//! turned down after (`pause_gate`), and the noise it leaves under the voice
//! is measured in the pauses and turned down band by band (`profile`).
//! Vocal cleanup can be made from the cleaned file in turn (`heard`).

mod profile;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use screenwide_transcriber::CLEAN_SAMPLE_RATE;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::profile::{Denoiser, Profile};
use super::analysis::{self, SpeechMap};
use super::choice_file;
use super::pause_gate::PauseGate;
use crate::transcription::runner::{decode_audio_stream, Samples, Transcriber};

/// Bumped when a cleaned file of an older make should be made again.
const FORMAT_VERSION: u16 = 5;

/// Whether the microphone's noise is taken out.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum NoiseReduction {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("noise-{stream}.json"))
}

/// The `stream`th track cleaned, in the project.
pub(super) fn cleaned_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("noise-{stream}.flac"))
}

/// What the project says about the `stream`th track's noise.
pub(super) fn choice(project_folder: &Path, stream: usize) -> NoiseReduction {
  choice_file::read(&choice_path(project_folder, stream), FORMAT_VERSION)
    .unwrap_or(NoiseReduction::Off)
}

pub(super) fn keep(
  project_folder: &Path,
  stream: usize,
  choice: NoiseReduction,
) -> Result<(), String> {
  forget_older(project_folder, stream)?;
  choice_file::write(&choice_path(project_folder, stream), FORMAT_VERSION, choice)
    .map_err(|error| format!("Could not keep the noise choice: {error}"))
}

/// Removes a file made by an older cleaning, with the cleaned-up voice made
/// from it, then marks the choice as kept by this one, so a file is only
/// ever there when it is current. The choice starts again from off, as it
/// does after any change of make.
fn forget_older(project_folder: &Path, stream: usize) -> Result<(), String> {
  let path = choice_path(project_folder, stream);
  if choice_file::read::<NoiseReduction>(&path, FORMAT_VERSION).is_some() {
    return Ok(());
  }
  let _ = std::fs::remove_file(cleaned_path(project_folder, stream));
  let _ = std::fs::remove_file(super::voice::cleaned_path(project_folder, stream, true));
  choice_file::write(&path, FORMAT_VERSION, NoiseReduction::Off)
    .map_err(|error| format!("Could not keep the noise choice: {error}"))
}

/// Whether the `stream`th track has been cleaned, by the current cleaning.
pub(super) fn is_made(project_folder: &Path, stream: usize) -> bool {
  choice_file::read::<NoiseReduction>(&choice_path(project_folder, stream), FORMAT_VERSION)
    .is_some()
    && cleaned_path(project_folder, stream).is_file()
}

/// How much of the progress bar listening for speech takes, before cleaning;
/// listening runs many times faster.
const LISTEN_SHARE: f32 = 0.1;
/// Samples read and written at a time while the cleaned track is encoded.
const ENCODE_CHUNK: usize = 1 << 16;

/// Cleans the `stream`th track of `movie` into the project, unless a file of
/// this make is there already, telling `progress` how far it has got. The
/// track's pauses are turned down by what the speech model at `model`
/// hears, which Remove silences listens for too.
pub(super) fn prepare(
  movie: &Path,
  stream: usize,
  project_folder: &Path,
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  forget_older(project_folder, stream)?;
  if is_made(project_folder, stream) {
    return Ok(());
  }
  let destination = cleaned_path(project_folder, stream);
  // Said before the track is read, so the editor shows its bar only when
  // there is cleaning to wait for, and shows it from the start.
  progress(0.0);
  let speech = analysis::speech_map(movie, stream, project_folder, model, &mut |fraction| {
    progress(fraction * LISTEN_SHARE);
  })?;
  let noisy = decode_audio_stream(movie, stream, CLEAN_SAMPLE_RATE)?;
  let profile = measure(&noisy, &speech)?;
  let clean = Samples::scratch();
  let gain = Transcriber::start()?.clean_speech(&noisy, &clean, &mut |fraction| {
    progress(LISTEN_SHARE + fraction * (1.0 - LISTEN_SHARE));
  })?;
  drop(noisy);

  // Written beside the destination and moved over it whole, so a cleaning cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  if let Err(error) = encode(&clean, gain, &speech, profile, &partial) {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the cleaned microphone: {error}"))?;
  // Vocal cleanup made from the file this replaces would no longer match it.
  let _ = std::fs::remove_file(super::voice::cleaned_path(project_folder, stream, true));
  Ok(())
}

/// The noise in the pauses of the microphone as recorded.
fn measure(noisy: &Samples, speech: &SpeechMap) -> Result<Option<Profile>, String> {
  let mut reader =
    File::open(noisy.path()).map_err(|error| format!("Could not read the microphone: {error}"))?;
  let length = reader
    .metadata()
    .map_err(|error| format!("Could not read the microphone: {error}"))?
    .len()
    / 4;
  Profile::measure(&mut reader, length, speech, CLEAN_SAMPLE_RATE)
}

/// Writes the cleaned samples in `clean` to `output` with the noise `profile`
/// measured turned down, the pauses turned down and `gain` applied, streamed
/// through FFmpeg so an hour of audio is never held in memory at once.
fn encode(
  clean: &Samples,
  gain: f32,
  speech: &SpeechMap,
  profile: Option<Profile>,
  output: &Path,
) -> Result<(), String> {
  let length = std::fs::metadata(clean.path())
    .map_err(|error| format!("Could not read the cleaned microphone: {error}"))?
    .len()
    / 4;
  let mut gate = PauseGate::new(speech, CLEAN_SAMPLE_RATE, length);
  let mut child = crate::editor::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-f", "f32le", "-ar"])
    .arg(CLEAN_SAMPLE_RATE.to_string())
    .args(["-ac", "1", "-i", "pipe:0"])
    // Back at the loudness it was recorded at, which the enhancer lowers by
    // taking the room's echo off with the noise.
    .args(["-af", &format!("volume={gain:.4}")])
    .args(["-c:a", "flac", "-sample_fmt", "s16"])
    .arg(output)
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  let mut input = child
    .stdin
    .take()
    .ok_or_else(|| "FFmpeg did not take the cleaned microphone".to_owned())?;
  let mut reader = File::open(clean.path())
    .map_err(|error| format!("Could not read the cleaned microphone: {error}"))?;
  let mut bytes = vec![0_u8; ENCODE_CHUNK * 4];
  let mut samples = Vec::with_capacity(ENCODE_CHUNK);
  let mut denoised = Vec::with_capacity(ENCODE_CHUNK);
  let mut denoiser = profile.map(Denoiser::new);
  let mut offset = 0_u64;
  let mut send = |samples: &mut Vec<f32>, bytes: &mut Vec<u8>| {
    gate.apply(offset, samples);
    offset += samples.len() as u64;
    bytes.clear();
    for sample in samples.iter() {
      bytes.extend_from_slice(&sample.to_le_bytes());
    }
    input.write_all(bytes)
  };
  // A write that fails means FFmpeg stopped, and what it says about why is
  // read below, so the error here adds nothing.
  let mut written = Ok(());
  let mut sending = Vec::with_capacity(ENCODE_CHUNK * 4);
  loop {
    let read = fill(&mut reader, &mut bytes)?;
    if read < 4 {
      break;
    }
    samples.clear();
    samples.extend(
      bytes[..read]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|sample| f32::from_le_bytes(*sample)),
    );
    let ready = match &mut denoiser {
      Some(denoiser) => {
        denoiser.feed(&samples, &mut denoised);
        &mut denoised
      }
      None => &mut samples,
    };
    written = send(ready, &mut sending);
    if written.is_err() {
      break;
    }
  }
  if let (Ok(()), Some(denoiser)) = (&written, &mut denoiser) {
    denoiser.finish(&mut denoised);
    written = send(&mut denoised, &mut sending);
  }
  drop(input);
  let finished = child
    .wait_with_output()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  if !finished.status.success() || written.is_err() {
    return Err(format!(
      "Could not write the cleaned microphone: {}",
      String::from_utf8_lossy(&finished.stderr).trim()
    ));
  }
  Ok(())
}

/// Reads from `reader` until `buffer` is full or the file ends, returning how
/// many bytes it holds.
fn fill(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize, String> {
  let mut filled = 0;
  while filled < buffer.len() {
    match reader.read(&mut buffer[filled..]) {
      Ok(0) => break,
      Ok(read) => filled += read,
      Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
      Err(error) => return Err(format!("Could not read the microphone: {error}")),
    }
  }
  Ok(filled)
}
