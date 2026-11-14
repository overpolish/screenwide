// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a song is sung, heard from its singing separated from the rest
//! (`singing`) in a few pieces of it, and kept with it. The separation
//! model leaves a trace of the instruments in the singing, tens of decibels
//! under them; where someone sings, the singing stands within a few
//! decibels of the rest, or above it. A second counts as sung where the
//! singing is no further under the rest than `SUNG_UNDER_REST_DB`, and a
//! piece as sung where `SUNG_SECONDS` of its seconds are, so a lead
//! instrument the model takes for a voice for a moment does not make an
//! instrumental song sung.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::process::Stdio;

use super::music::{Music, Vocals};
use super::singing::{rest_path, singing_path, PIECE_MS};
use super::{directory, is_name, read_kept, write_kept, Sound};

/// The rate the parts are read at, in Hz.
const READ_RATE: usize = 8_000;
/// How far under the rest the singing may be and still be singing, in dB.
/// In six sung songs, the singing sat within 10 dB of the rest through most
/// of each piece heard; in four instrumental ones it stayed 20 dB and more
/// under, but for two seconds of one, which a lead instrument rose to 6 dB
/// under.
const SUNG_UNDER_REST_DB: f32 = 12.0;
/// How many seconds of a piece must be sung for it to count.
const SUNG_SECONDS: usize = 3;
/// Below this, as root mean square of the song, a second is silence and
/// says nothing either way: -50 dBFS.
const SILENT_RMS: f32 = 0.003;

/// Whether the song in the sound file `sound` is sung, from its `pieces`,
/// which must be separated already.
pub fn is_sung(sound: &Path, pieces: &BTreeSet<u64>) -> Result<bool, String> {
  let singing = mono(&singing_path(sound))?;
  let rest = mono(&rest_path(sound))?;
  let piece_samples = PIECE_MS as usize * READ_RATE / 1_000;
  Ok(pieces.iter().any(|&piece| {
    let start = piece as usize * piece_samples;
    let seconds = (start..start + piece_samples)
      .step_by(READ_RATE)
      .filter(|&second| is_sung_second(&singing, &rest, second))
      .count();
    seconds >= SUNG_SECONDS
  }))
}

fn is_sung_second(singing: &[f32], rest: &[f32], start: usize) -> bool {
  let rms = |samples: &[f32], start: usize| {
    let end = (start + READ_RATE).min(samples.len());
    let part = samples.get(start..end).unwrap_or_default();
    if part.is_empty() {
      return 0.0;
    }
    (part.iter().map(|sample| sample.powi(2)).sum::<f32>() / part.len() as f32).sqrt()
  };
  let (sung, played) = (rms(singing, start), rms(rest, start));
  if sung.hypot(played) < SILENT_RMS {
    return false;
  }
  20.0 * (sung / played.max(f32::EPSILON)).log10() >= -SUNG_UNDER_REST_DB
}

/// The sound file `path` as mono samples at `READ_RATE`.
fn mono(path: &Path) -> Result<Vec<f32>, String> {
  let mut child = screenwide_sidecar::ffmpeg_command()
    .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
    .arg(path)
    .args(["-map", "0:a:0", "-vn", "-af"])
    .arg(format!(
      "aresample={READ_RATE},aformat=sample_fmts=flt:channel_layouts=mono"
    ))
    .args(["-f", "f32le", "pipe:1"])
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  let mut bytes = Vec::new();
  if let Some(mut stdout) = child.stdout.take() {
    stdout
      .read_to_end(&mut bytes)
      .map_err(|error| format!("Could not read the separated song: {error}"))?;
  }
  let status = child
    .wait()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  if !status.success() {
    return Err(format!("FFmpeg could not read {}", path.display()));
  }
  Ok(
    bytes
      .as_chunks::<4>()
      .0
      .iter()
      .map(|chunk| f32::from_le_bytes(*chunk))
      .collect(),
  )
}

/// Keeps with the library's sound `name` whether it is sung, as heard.
pub fn keep_heard_vocals(name: &str, vocals: Vocals) -> Result<(), String> {
  let folder = directory()?;
  let sound = is_name(name)
    .then(|| read_kept(folder, name))
    .flatten()
    .ok_or_else(|| "There is no such sound".to_owned())?;
  let music = Music {
    vocals: Some(vocals),
    ..sound.music.unwrap_or_default()
  };
  write_kept(
    folder,
    &Sound {
      music: Some(music),
      ..sound
    },
  )
}
