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
//!
//! The model runs on the GPU where it can: on Windows in parts through
//! DirectML (`staged`), elsewhere whole. A GPU that fails, or gives an
//! answer no singing could give, moves the work to the CPU, which redoes
//! that chunk and the rest, so ducking never fails or goes wrong because of
//! the GPU.

#[cfg(windows)]
mod staged;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;

use crate::session::on_cpu;

/// Samples of each side the model hears at once: 7.8 s at its rate.
const SEGMENT: usize = 343_980;
const OVERLAP: usize = SEGMENT / 4;
const STRIDE: usize = SEGMENT - OVERLAP;
const SIDES: usize = 2;
/// Where the singing is among the model's stems: drums, bass, other, vocals.
const VOCALS: usize = 3;
/// The feature's name in what the helper reports.
const WHAT: &str = "Studio ducking";

/// Where the model runs.
enum Runner {
  #[cfg(windows)]
  Staged(staged::Staged),
  #[cfg(not(windows))]
  Gpu(Session),
  Cpu(Session),
}

impl Runner {
  fn new(model: &Path) -> Result<Self, String> {
    #[cfg(windows)]
    let gpu = match std::fs::read(model)
      .map_err(|error| error.to_string())
      .and_then(|bytes| staged::Staged::new(&bytes))
    {
      Ok(staged) => {
        eprintln!("{WHAT} runs HT Demucs on the GPU through DirectML, in parts");
        Some(Self::Staged(staged))
      }
      Err(error) => {
        eprintln!("{WHAT} runs on the CPU: the GPU could not take it: {error}");
        None
      }
    };
    #[cfg(not(windows))]
    let gpu = crate::session::gpu_session(model, WHAT).map(Self::Gpu);
    match gpu {
      Some(runner) => Ok(runner),
      None => on_cpu(model, WHAT).map(Self::Cpu),
    }
  }

  /// The singing in one planar segment. `model` reloads it on the CPU when
  /// the GPU's answer cannot be used.
  fn singing(&mut self, model: &Path, planar: Vec<f32>) -> Result<Vec<f32>, String> {
    if !matches!(self, Self::Cpu(_)) {
      let peak = planar
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
      match self
        .on_gpu(planar.clone())
        .and_then(|singing| plausible(singing, peak))
      {
        Ok(singing) => return Ok(singing),
        Err(error) => {
          eprintln!("{WHAT} moves to the CPU: {error}");
          *self = Self::Cpu(on_cpu(model, WHAT)?);
        }
      }
    }
    let Self::Cpu(session) = self else {
      unreachable!("the GPU runner moves to the CPU before this")
    };
    whole(session, planar).map_err(|error| format!("{WHAT} could not run: {error}"))
  }

  /// The GPU's answer for one planar segment, not yet checked.
  fn on_gpu(&mut self, planar: Vec<f32>) -> Result<Vec<f32>, String> {
    match self {
      #[cfg(windows)]
      Self::Staged(staged) => {
        let input = Tensor::from_array(([1_usize, SIDES, SEGMENT], planar))
          .map_err(|error| error.to_string())?;
        let mut outputs = staged.run(input.into_dyn())?;
        let stems = outputs.remove("stems").ok_or("the model made no stems")?;
        let (_, stems) = stems
          .try_extract_tensor::<f32>()
          .map_err(|error| error.to_string())?;
        vocals(stems)
      }
      #[cfg(not(windows))]
      Self::Gpu(session) => whole(session, planar),
      Self::Cpu(_) => unreachable!("the CPU runner is never asked for the GPU's answer"),
    }
  }
}

/// The singing in one planar segment by the whole model in `session`.
fn whole(session: &mut Session, planar: Vec<f32>) -> Result<Vec<f32>, String> {
  let failed = |error: ort::Error| error.to_string();
  let input = Tensor::from_array(([1_usize, SIDES, SEGMENT], planar)).map_err(failed)?;
  let outputs = session.run(ort::inputs!["mix" => input]).map_err(failed)?;
  let (_, stems) = outputs["stems"]
    .try_extract_tensor::<f32>()
    .map_err(failed)?;
  vocals(stems)
}

/// The singing among the model's four stems.
fn vocals(stems: &[f32]) -> Result<Vec<f32>, String> {
  let from = VOCALS * SIDES * SEGMENT;
  stems
    .get(from..from + SIDES * SEGMENT)
    .map(<[f32]>::to_vec)
    .ok_or_else(|| "the model made too few stems".to_owned())
}

/// `singing` unless no singing in audio peaking at `peak` could be it: a
/// separated voice is never far louder than the mix it came from, and a
/// GPU that went wrong answers with numbers thousands of times too large.
fn plausible(singing: Vec<f32>, peak: f32) -> Result<Vec<f32>, String> {
  let limit = 2.0 * peak + 0.01;
  match singing
    .iter()
    .find(|sample| !sample.is_finite() || sample.abs() > limit)
  {
    Some(sample) => Err(format!(
      "the GPU answered {sample} for audio peaking at {peak}"
    )),
    None => Ok(singing),
  }
}

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
  let mut runner = Runner::new(model)?;
  overlap_add(
    &mut reader,
    &mut writer,
    total,
    &mut |planar| runner.singing(model, planar),
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
