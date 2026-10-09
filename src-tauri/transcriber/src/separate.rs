// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio ducking: the singing in the system audio, found by HT Demucs, a
//! music source separation model from Meta, so the app can move the singer
//! out of the voice's way and leave the music playing.
//!
//! The model hears 7.8 s at a time. Chunks overlap by a quarter and are
//! faded across inside the overlap, as its reference inference does, and
//! the audio is read and written as it goes, so an hour of it is never held
//! in memory at once.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use ort::value::Tensor;

use crate::session::session;

/// Samples of each side the model hears at once: 7.8 s at its rate.
const SEGMENT: usize = 343_980;
const OVERLAP: usize = SEGMENT / 4;
const STRIDE: usize = SEGMENT - OVERLAP;
const SIDES: usize = 2;
/// Where the singing is among the model's stems: drums, bass, other, vocals.
const VOCALS: usize = 3;
/// The feature's name in what the helper reports.
const WHAT: &str = "Studio ducking";

/// Writes the singing in the stereo samples at `audio`, at the separation
/// rate, to `output` in the same form, by the model at `model`. Tells
/// `progress` how far it has got.
pub fn separate_vocals(
  model: &Path,
  audio: &Path,
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the audio: {error}");
  let bytes = std::fs::metadata(audio).map_err(unreadable)?.len();
  let total = usize::try_from(bytes / (4 * SIDES as u64)).unwrap_or(usize::MAX);
  let mut reader = BufReader::new(File::open(audio).map_err(unreadable)?);
  let mut writer = BufWriter::new(
    File::create(output).map_err(|error| format!("Could not write the singing: {error}"))?,
  );
  let mut session = session(model, WHAT)?;
  let failed = |error: ort::Error| format!("{WHAT} could not run: {error}");
  overlap_add(
    &mut reader,
    &mut writer,
    total,
    &mut |planar| {
      let input = Tensor::from_array(([1_usize, SIDES, SEGMENT], planar)).map_err(failed)?;
      let outputs = session.run(ort::inputs!["mix" => input]).map_err(failed)?;
      let (_, stems) = outputs["stems"]
        .try_extract_tensor::<f32>()
        .map_err(failed)?;
      let from = VOCALS * SIDES * SEGMENT;
      Ok(stems[from..from + SIDES * SEGMENT].to_vec())
    },
    progress,
  )?;
  writer
    .flush()
    .map_err(|error| format!("Could not write the singing: {error}"))
}

/// Runs `separate` over the `total` interleaved stereo frames from `reader`
/// a segment at a time, each segment planar, left side then right, padded
/// with silence to [`SEGMENT`], and writes what it gives back, faded across
/// where segments overlap, to `writer` interleaved.
fn overlap_add(
  reader: &mut impl Read,
  writer: &mut impl Write,
  total: usize,
  separate: &mut dyn FnMut(Vec<f32>) -> Result<Vec<f32>, String>,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the audio: {error}");
  let weights = weights();
  // The frames from `start` on, interleaved: those read, and the sums and
  // weights the segments so far have added to them.
  let mut mix: Vec<f32> = Vec::with_capacity(SEGMENT * SIDES);
  let mut sums = vec![0.0_f32; SEGMENT * SIDES];
  let mut weighed = vec![0.0_f32; SEGMENT];
  let mut start = 0;
  while start < total {
    let length = SEGMENT.min(total - start);
    read_into(reader, length * SIDES - mix.len(), &mut mix).map_err(unreadable)?;
    let mut planar = vec![0.0_f32; SIDES * SEGMENT];
    for (frame, sides) in mix.as_chunks::<SIDES>().0.iter().enumerate() {
      for (side, &sample) in sides.iter().enumerate() {
        planar[side * SEGMENT + frame] = sample;
      }
    }
    let singing = separate(planar)?;
    for (frame, &weight) in weights[..length].iter().enumerate() {
      for side in 0..SIDES {
        sums[frame * SIDES + side] += singing[side * SEGMENT + frame] * weight;
      }
      weighed[frame] += weight;
    }
    // Frames before the next segment's start have all they will get.
    let settled = if start + length >= total {
      length
    } else {
      STRIDE
    };
    write_settled(writer, &sums[..settled * SIDES], &weighed[..settled])?;
    sums.drain(..settled * SIDES);
    sums.resize(SEGMENT * SIDES, 0.0);
    weighed.drain(..settled);
    weighed.resize(SEGMENT, 0.0);
    mix.drain(..settled.min(length) * SIDES);
    start += settled;
    #[expect(clippy::cast_precision_loss, reason = "a fraction")]
    progress(start as f32 / total as f32);
  }
  Ok(())
}

/// How much each frame of a segment counts: fading in and out across the
/// overlaps, and never quite nothing, so the first and last frames of the
/// audio, which only one segment reaches, keep what it made of them.
fn weights() -> Vec<f32> {
  #[expect(clippy::cast_precision_loss, reason = "a fade")]
  let fade = |frame: usize| (frame + 1) as f32 / (OVERLAP + 1) as f32;
  (0..SEGMENT)
    .map(|frame| fade(frame.min(SEGMENT - 1 - frame)).min(1.0))
    .collect()
}

fn write_settled(writer: &mut impl Write, sums: &[f32], weighed: &[f32]) -> Result<(), String> {
  let bytes: Vec<u8> = sums
    .as_chunks::<SIDES>()
    .0
    .iter()
    .zip(weighed)
    .flat_map(|(sides, &weight)| sides.iter().map(move |sum| sum / weight))
    .flat_map(f32::to_le_bytes)
    .collect();
  writer
    .write_all(&bytes)
    .map_err(|error| format!("Could not write the singing: {error}"))
}

/// Reads `count` samples from `reader` onto the end of `into`.
fn read_into(reader: &mut impl Read, count: usize, into: &mut Vec<f32>) -> std::io::Result<()> {
  let mut bytes = vec![0_u8; count * 4];
  reader.read_exact(&mut bytes)?;
  into.extend(
    bytes
      .as_chunks::<4>()
      .0
      .iter()
      .map(|&sample| f32::from_le_bytes(sample)),
  );
  Ok(())
}
