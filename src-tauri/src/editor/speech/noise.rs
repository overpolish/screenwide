// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reduce noise: the microphone track cleaned once into a file of its own in
//! the project, which preview and export play in its place while the switch
//! is on. The cleaning is DeepFilterNet's, run in the transcriber program: a
//! speech enhancer that keeps the voice and takes out what is not, steady
//! hiss and passing clicks alike. What it lets through between words is
//! turned down after (`pause_gate`). The choice is kept beside the file, so
//! it holds when the project opens again and the export reads the same
//! answer as the preview.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use screenwide_transcriber::CLEAN_SAMPLE_RATE;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::analysis::{self, SpeechMap};
use super::pause_gate::PauseGate;
use crate::transcription::runner::{decode_audio_stream, Samples, Transcriber};

/// Bumped when a cleaned file of an older make should be made again.
const FORMAT_VERSION: u16 = 3;

/// Whether the microphone's noise is taken out.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum NoiseReduction {
  #[default]
  Off,
  On,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Kept {
  version: u16,
  choice: NoiseReduction,
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("noise-{stream}.json"))
}

/// The `stream`th track cleaned, in the project.
pub(super) fn cleaned_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("noise-{stream}.flac"))
}

fn kept(project_folder: &Path, stream: usize) -> Option<Kept> {
  std::fs::read(choice_path(project_folder, stream))
    .ok()
    .and_then(|bytes| serde_json::from_slice::<Kept>(&bytes).ok())
}

/// What the project says about the `stream`th track's noise.
pub(super) fn choice(project_folder: &Path, stream: usize) -> NoiseReduction {
  kept(project_folder, stream)
    .filter(|kept| kept.version == FORMAT_VERSION)
    .map_or(NoiseReduction::Off, |kept| kept.choice)
}

pub(super) fn keep(
  project_folder: &Path,
  stream: usize,
  choice: NoiseReduction,
) -> Result<(), String> {
  let kept = Kept {
    choice,
    version: FORMAT_VERSION,
  };
  std::fs::write(
    choice_path(project_folder, stream),
    serde_json::to_vec(&kept).map_err(|error| error.to_string())?,
  )
  .map_err(|error| format!("Could not keep the noise choice: {error}"))
}

/// The tracks of the project in `project_folder` to play cleaned, with the
/// files they are cleaned into: those with the switch on whose file is there.
pub(crate) fn cleaned_tracks(project_folder: &Path, streams: &[usize]) -> Vec<(usize, PathBuf)> {
  cleaned_files(project_folder, streams)
    .into_iter()
    .filter(|(stream, _)| choice(project_folder, *stream) == NoiseReduction::On)
    .collect()
}

/// The tracks of the project in `project_folder` that have been cleaned,
/// whether or not they are heard so, with their files. A file made before the
/// current cleaning is not one of them.
pub(crate) fn cleaned_files(project_folder: &Path, streams: &[usize]) -> Vec<(usize, PathBuf)> {
  streams
    .iter()
    .filter(|&&stream| {
      kept(project_folder, stream).is_some_and(|kept| kept.version == FORMAT_VERSION)
    })
    .map(|&stream| (stream, cleaned_path(project_folder, stream)))
    .filter(|(_, path)| path.is_file())
    .collect()
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
  let destination = cleaned_path(project_folder, stream);
  if destination.is_file()
    && kept(project_folder, stream).is_some_and(|kept| kept.version == FORMAT_VERSION)
  {
    return Ok(());
  }
  // Said before the track is read, so the editor shows its bar only when
  // there is cleaning to wait for, and shows it from the start.
  progress(0.0);
  let speech = analysis::speech_map(movie, stream, project_folder, model, &mut |fraction| {
    progress(fraction * LISTEN_SHARE);
  })?;
  let noisy = decode_audio_stream(movie, stream, CLEAN_SAMPLE_RATE)?;
  let clean = Samples::scratch();
  let gain = Transcriber::start()?.clean_speech(&noisy, &clean, &mut |fraction| {
    progress(LISTEN_SHARE + fraction * (1.0 - LISTEN_SHARE));
  })?;
  drop(noisy);

  // Written beside the destination and moved over it whole, so a cleaning cut
  // short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  if let Err(error) = encode(&clean, gain, &speech, &partial) {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the cleaned microphone: {error}"))
}

/// Writes the cleaned samples in `clean` to `output` with their pauses
/// turned down and `gain` applied, streamed through FFmpeg so an hour of
/// audio is never held in memory at once.
fn encode(clean: &Samples, gain: f32, speech: &SpeechMap, output: &Path) -> Result<(), String> {
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
  let mut offset = 0_u64;
  // A write that fails means FFmpeg stopped, and what it says about why is
  // read below, so the error here adds nothing.
  let mut written = Ok(());
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
    gate.apply(offset, &mut samples);
    offset += samples.len() as u64;
    for (sample, out) in samples.iter().zip(bytes.as_chunks_mut::<4>().0) {
      *out = sample.to_le_bytes();
    }
    written = input.write_all(&bytes[..samples.len() * 4]);
    if written.is_err() {
      break;
    }
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
fn fill(reader: &mut File, buffer: &mut [u8]) -> Result<usize, String> {
  let mut filled = 0;
  while filled < buffer.len() {
    match reader.read(&mut buffer[filled..]) {
      Ok(0) => break,
      Ok(read) => filled += read,
      Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
      Err(error) => return Err(format!("Could not read the cleaned microphone: {error}")),
    }
  }
  Ok(filled)
}
