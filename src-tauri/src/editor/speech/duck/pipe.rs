// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The tracks streamed through FFmpeg: read as samples a few seconds at a
//! time, and the system audio written back out as it is made, so an hour of
//! audio is never held in memory at once.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};

use screenwide_transcriber::CLEAN_SAMPLE_RATE as RATE;

use super::framer::{Framer, SIZE};
use super::plan::Plan;

/// Samples read at a time, across all channels.
const CHUNK: usize = 1 << 16;
/// The system audio keeps both its sides.
const SIDES: usize = 2;

/// FFmpeg reading the `stream`th audio track of `movie` as `channels`
/// interleaved channels at the cleanup's rate, lined up with the recording's
/// start as every other track is.
fn reader(movie: &Path, stream: usize, channels: usize) -> Command {
  let mut command = crate::editor::ffmpeg_command();
  command
    .args(["-nostdin", "-loglevel", "error", "-i"])
    .arg(movie)
    .args(["-map", &format!("0:a:{stream}"), "-vn"])
    .args(["-af", "aresample=async=1:first_pts=0"])
    .args(["-ac", &channels.to_string(), "-ar", &RATE.to_string()])
    .args(["-f", "f32le", "pipe:1"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  command
}

/// Hands `on_samples` the `stream`th track of `movie`, a chunk at a time.
pub(super) fn decode(
  movie: &Path,
  stream: usize,
  channels: usize,
  on_samples: &mut dyn FnMut(&[f32]),
) -> Result<(), String> {
  let mut child = reader(movie, stream, channels)
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  let mut stdout = take_stdout(&mut child)?;
  let mut samples = Vec::with_capacity(CHUNK);
  let mut bytes = vec![0_u8; CHUNK * 4];
  loop {
    let read = fill(&mut stdout, &mut bytes)?;
    if read < 4 {
      break;
    }
    to_samples(&bytes[..read], &mut samples);
    on_samples(&samples);
  }
  finish(child, "read the audio")
}

/// Writes the `stream`th track of `movie` to `output` with each frame turned
/// down as `plan` says, telling `on_read` how many samples of each side it
/// has read.
pub(super) fn make_way(
  movie: &Path,
  stream: usize,
  output: &Path,
  plan: &mut Plan,
  on_read: &mut dyn FnMut(usize),
) -> Result<(), String> {
  let mut decoding = reader(movie, stream, SIDES)
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  let mut stdout = take_stdout(&mut decoding)?;
  let mut encoding = crate::editor::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-f", "f32le", "-ar"])
    .arg(RATE.to_string())
    .args(["-ac", &SIDES.to_string(), "-i", "pipe:0"])
    .args(["-c:a", "flac", "-sample_fmt", "s16"])
    .arg(output)
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  let mut input = encoding
    .stdin
    .take()
    .ok_or_else(|| "FFmpeg did not take the system audio".to_owned())?;
  let mut sides = [Framer::new(), Framer::new()];
  let mut sent = Sides::default();
  let mut bytes = vec![0_u8; CHUNK * 4];
  let mut samples = Vec::with_capacity(CHUNK);
  let mut written = Ok(());
  loop {
    let read = fill(&mut stdout, &mut bytes)?;
    if read < 4 * SIDES {
      break;
    }
    to_samples(&bytes[..read - read % (4 * SIDES)], &mut samples);
    sent.feed(&samples, &mut sides, plan, true);
    on_read(sent.read);
    written = sent.write(&mut input);
    if written.is_err() {
      break;
    }
  }
  if written.is_ok() {
    // Silence after the end lets the last frames finish.
    let tail = vec![0.0; SIZE * SIDES];
    sent.feed(&tail, &mut sides, plan, false);
    written = sent.write(&mut input);
  }
  drop(input);
  finish(decoding, "read the system audio")?;
  let finished = encoding
    .wait_with_output()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  if !finished.status.success() || written.is_err() {
    return Err(format!(
      "Could not write the system audio: {}",
      String::from_utf8_lossy(&finished.stderr).trim()
    ));
  }
  Ok(())
}

/// The two sides on their way through their framers.
#[derive(Default)]
struct Sides {
  /// Samples of each side read from the track, which is how many go out.
  read: usize,
  /// Samples of each side that came out of the framers, the first
  /// `SIZE - HOP` of which lie before the start.
  made: usize,
  split: [Vec<f32>; SIDES],
  out: [Vec<f32>; SIDES],
  bytes: Vec<u8>,
}

impl Sides {
  /// Feeds interleaved `samples` through `sides`, turned down as `plan`
  /// says, counting them as read from the track when `from_track`.
  fn feed(
    &mut self,
    samples: &[f32],
    sides: &mut [Framer; SIDES],
    plan: &mut Plan,
    from_track: bool,
  ) {
    for (side, split) in self.split.iter_mut().enumerate() {
      split.clear();
      split.extend(samples.iter().skip(side).step_by(SIDES));
    }
    if from_track {
      self.read += self.split[0].len();
    }
    for ((framer, split), out) in sides.iter_mut().zip(&self.split).zip(&mut self.out) {
      framer.feed(split, Some(out), &mut |frame, spectrum| {
        plan.apply(frame, spectrum)
      });
    }
  }

  /// Writes what has come out of both sides since last time, leaving out
  /// what lies before the start and after the end.
  fn write(&mut self, input: &mut impl Write) -> Result<(), String> {
    let latency = SIZE - super::framer::HOP;
    let length = self.out[0].len().min(self.out[1].len());
    self.bytes.clear();
    for at in 0..length {
      let made = self.made + at;
      if made < latency || made - latency >= self.read {
        continue;
      }
      for out in &self.out {
        self.bytes.extend_from_slice(&out[at].to_le_bytes());
      }
    }
    self.made += length;
    for out in &mut self.out {
      out.drain(..length);
    }
    input
      .write_all(&self.bytes)
      .map_err(|error| error.to_string())
  }
}

fn take_stdout(child: &mut Child) -> Result<ChildStdout, String> {
  child
    .stdout
    .take()
    .ok_or_else(|| "FFmpeg did not hand over the audio".to_owned())
}

fn to_samples(bytes: &[u8], samples: &mut Vec<f32>) {
  samples.clear();
  samples.extend(
    bytes
      .as_chunks::<4>()
      .0
      .iter()
      .map(|bytes| f32::from_le_bytes(*bytes)),
  );
}

/// Reads from `reader` until `buffer` is full or the stream ends, returning
/// how many bytes it holds.
fn fill(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize, String> {
  let mut filled = 0;
  while filled < buffer.len() {
    match reader.read(&mut buffer[filled..]) {
      Ok(0) => break,
      Ok(read) => filled += read,
      Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
      Err(error) => return Err(format!("Could not read the audio: {error}")),
    }
  }
  Ok(filled)
}

/// Waits for a reading FFmpeg, failing with what it said if it failed.
fn finish(child: Child, doing: &str) -> Result<(), String> {
  let finished = child
    .wait_with_output()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  if finished.status.success() {
    Ok(())
  } else {
    Err(format!(
      "Could not {doing}: {}",
      String::from_utf8_lossy(&finished.stderr).trim()
    ))
  }
}
