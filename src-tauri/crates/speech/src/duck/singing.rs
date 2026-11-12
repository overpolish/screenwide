// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The singing in music, separated by the transcriber with the music
//! separation model: in the system audio, for Studio ducking to move out of
//! the voice's way while the music stays, and in songs from the sound
//! library, a stretch at a time, written into the song's singing file.

use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use screenwide_transcriber::SEPARATE_SAMPLE_RATE;

use crate::runner::samples::{decode_stereo_stream, decode_stereo_stretch, Samples};
use crate::runner::Transcriber;

const SIDES: usize = 2;
/// Frames copied at a time while the singing file is rewritten.
const BLOCK: usize = 1 << 16;

/// The singing in the `stream`th track of `movie`, both sides interleaved at
/// the protocol's separation rate, found by the model at `model`, telling
/// `progress` how far it has got.
pub(super) fn separate(
  movie: &Path,
  stream: usize,
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<Samples, String> {
  let mix = decode_stereo_stream(movie, stream, SEPARATE_SAMPLE_RATE)?;
  let singing = Samples::scratch();
  Transcriber::start()?.separate_vocals(model, &mix, &singing, progress)?;
  Ok(singing)
}

/// Separates frames `from` to `to` of `sound` with the model at `model` and
/// writes them into the FLACs at `destinations`: its singing, and the rest
/// of it, worked out from the same reading of the song so the two always
/// add up to it, both at `level` of their own loudness, so a part louder
/// than full scale still fits a FLAC. Each keeps what it held elsewhere and
/// is silent where nothing was separated yet, and is written beside its
/// destination and moved over it whole.
pub(super) fn into_song(
  sound: &Path,
  model: PathBuf,
  (from, to): (u64, u64),
  (singing_to, rest_to, level): (&Path, &Path, f32),
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let mix = decode_stereo_stretch(sound, SEPARATE_SAMPLE_RATE, (from, to))?;
  let singing = Samples::scratch();
  Transcriber::start()?.separate_vocals(model, &mix, &singing, progress)?;
  let singing = singing.read()?;
  let rest: Vec<f32> = mix
    .read()?
    .iter()
    .zip(&singing)
    .map(|(mix, singing)| (mix - singing) * level)
    .collect();
  let singing: Vec<f32> = singing.iter().map(|sample| sample * level).collect();
  for (part, destination) in [(&singing, singing_to), (&rest, rest_to)] {
    let partial = destination.with_extension("partial.flac");
    if let Err(error) = splice(destination, part, from, &partial) {
      let _ = std::fs::remove_file(&partial);
      return Err(format!(
        "Could not keep the singing separated from the song: {error}"
      ));
    }
    std::fs::rename(&partial, destination)
      .map_err(|error| format!("Could not keep the singing separated from the song: {error}"))?;
  }
  Ok(())
}

fn ffmpeg(arguments: &[&str]) -> Command {
  let mut command = screenwide_sidecar::ffmpeg_command();
  command
    .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"])
    .args(arguments)
    .stderr(Stdio::null());
  command
}

/// Writes to `output` the part in `existing`, where that file is, with
/// `new` laid over it from frame `from` on: as long as the longer of the
/// two, silent where neither has anything.
fn splice(existing: &Path, new: &[f32], from: u64, output: &Path) -> Result<(), String> {
  let rate = SEPARATE_SAMPLE_RATE.to_string();
  let raw = ["-f", "f32le", "-ar", rate.as_str(), "-ac", "2"];
  let mut decoder: Option<Child> = existing
    .is_file()
    .then(|| {
      ffmpeg(&[])
        .arg("-i")
        .arg(existing)
        .args(raw)
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .spawn()
    })
    .transpose()
    .map_err(|error| error.to_string())?;
  let mut held = decoder
    .as_mut()
    .and_then(|decoder| decoder.stdout.take())
    .map(BufReader::new);
  let mut encoder = ffmpeg(&raw)
    .args(["-i", "pipe:0", "-c:a", "flac"])
    .arg(output)
    .stdin(Stdio::piped())
    .spawn()
    .map_err(|error| error.to_string())?;
  let mut input = encoder.stdin.take().ok_or("FFmpeg took no input")?;
  let new_frames = new.len() / SIDES;
  let from = usize::try_from(from).map_err(|error| error.to_string())?;
  let mut at = 0;
  loop {
    let mut block = vec![0.0_f32; BLOCK * SIDES];
    let read = match held.as_mut() {
      Some(reader) => read_frames(reader, &mut block)?,
      None => 0,
    };
    let overlaid = (from + new_frames).saturating_sub(at).min(BLOCK);
    let frames = read.max(overlaid);
    if frames == 0 {
      break;
    }
    for frame in at.max(from)..(at + frames).min(from + new_frames) {
      let (into, out) = ((frame - at) * SIDES, (frame - from) * SIDES);
      block[into..into + SIDES].copy_from_slice(&new[out..out + SIDES]);
    }
    write_frames(&mut input, &block[..frames * SIDES])?;
    at += frames;
  }
  drop(input);
  if let Some(mut decoder) = decoder {
    let _ = decoder.wait();
  }
  let status = encoder.wait().map_err(|error| error.to_string())?;
  status
    .success()
    .then_some(())
    .ok_or_else(|| "FFmpeg could not write it".to_owned())
}

/// Fills `block` from `reader` as far as it goes, in whole frames, and says
/// how many it read: fewer than the block holds only at the end.
fn read_frames(reader: &mut impl Read, block: &mut [f32]) -> Result<usize, String> {
  let mut bytes = vec![0_u8; block.len() * 4];
  let mut filled = 0;
  while filled < bytes.len() {
    match reader.read(&mut bytes[filled..]) {
      Ok(0) => break,
      Ok(count) => filled += count,
      Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
      Err(error) => return Err(error.to_string()),
    }
  }
  let samples = filled / 4 / SIDES * SIDES;
  for (slot, bytes) in block.iter_mut().zip(bytes.as_chunks::<4>().0).take(samples) {
    *slot = f32::from_le_bytes(*bytes);
  }
  Ok(samples / SIDES)
}

fn write_frames(input: &mut ChildStdin, samples: &[f32]) -> Result<(), String> {
  let bytes: Vec<u8> = samples
    .iter()
    .flat_map(|sample| sample.to_le_bytes())
    .collect();
  input.write_all(&bytes).map_err(|error| error.to_string())
}
