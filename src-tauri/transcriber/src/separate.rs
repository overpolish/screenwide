// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio ducking and instrumental songs: the singing in music, found by
//! Kimberley Jensen's Mel-Band RoFormer, so the app can move the singer out
//! of the voice's way, or take the singing out of a song, and leave the
//! music playing.
//!
//! The model hears 8 s at a time, as a spectrum (`spectrum`), and answers
//! with the singing's. Chunks follow each other from the start of the audio
//! without overlapping, so each chunk's singing depends on that chunk alone:
//! a stretch of a song separated on its own, starting on a chunk's boundary,
//! comes out exactly as it does from the whole song, and a song can be
//! separated a piece at a time. Overlapping chunks faded across measure
//! about 0.1 dB better for twice the work. The audio is read and written as
//! it goes, so an hour of it is never held in memory at once.
//!
//! The model runs on the GPU where it can: WebGPU on macOS, DirectML on
//! Windows. A GPU that fails, or gives an answer no singing could give, moves
//! the work to the CPU, which redoes that chunk and the rest, so ducking
//! never fails or goes wrong because of the GPU.

mod spectrum;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;

use screenwide_transcriber::SEPARATE_CHUNK as CHUNK;

use crate::session::{gpu_session, on_cpu};
use spectrum::{Spectrum, BINS, FRAMES};

const SIDES: usize = 2;
/// The feature's name in what the helper reports.
const WHAT: &str = "Studio ducking";

/// Where the model runs.
enum Runner {
  Gpu(Session),
  Cpu(Session),
}

impl Runner {
  fn new(model: &Path) -> Result<Self, String> {
    match gpu_session(model, WHAT) {
      Some(session) => Ok(Self::Gpu(session)),
      None => on_cpu(model, WHAT).map(Self::Cpu),
    }
  }

  /// The singing in one planar chunk. `model` reloads it on the CPU when the
  /// GPU's answer cannot be used.
  fn singing(
    &mut self,
    model: &Path,
    spectrum: &mut Spectrum,
    planar: &[f32],
  ) -> Result<Vec<f32>, String> {
    let input = spectrum.analyse(planar);
    if let Self::Gpu(session) = self {
      let peak = planar
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
      match run(session, input.clone())
        .map(|singing| spectrum.synthesise(&singing))
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
    run(session, input)
      .map(|singing| spectrum.synthesise(&singing))
      .map_err(|error| format!("{WHAT} could not run: {error}"))
  }
}

/// The singing's spectrum for one chunk's, by the model in `session`.
fn run(session: &mut Session, input: Vec<f32>) -> Result<Vec<f32>, String> {
  let failed = |error: ort::Error| error.to_string();
  let input = Tensor::from_array(([1_usize, 2 * SIDES, BINS, FRAMES], input)).map_err(failed)?;
  let outputs = session
    .run(ort::inputs!["mix_spec" => input])
    .map_err(failed)?;
  let (_, singing) = outputs["stems_spec"]
    .try_extract_tensor::<f32>()
    .map_err(failed)?;
  Ok(singing.to_vec())
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
  let mut spectrum = Spectrum::new();
  chunked(
    &mut reader,
    &mut writer,
    total,
    &mut |planar| runner.singing(model, &mut spectrum, &planar),
    progress,
  )?;
  writer
    .flush()
    .map_err(|error| format!("Could not write the singing: {error}"))
}

/// Runs `separate` over the `total` interleaved stereo frames from `reader`
/// a chunk at a time, each chunk planar, left side then right, padded with
/// silence to [`CHUNK`], and writes what it gives back for the frames read
/// to `writer` interleaved.
fn chunked(
  reader: &mut impl Read,
  writer: &mut impl Write,
  total: usize,
  separate: &mut dyn FnMut(Vec<f32>) -> Result<Vec<f32>, String>,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the audio: {error}");
  let mut mix: Vec<f32> = Vec::with_capacity(CHUNK * SIDES);
  let mut start = 0;
  while start < total {
    let length = CHUNK.min(total - start);
    mix.clear();
    read_into(reader, length * SIDES, &mut mix).map_err(unreadable)?;
    let mut planar = vec![0.0_f32; SIDES * CHUNK];
    for (frame, sides) in mix.as_chunks::<SIDES>().0.iter().enumerate() {
      for (side, &sample) in sides.iter().enumerate() {
        planar[side * CHUNK + frame] = sample;
      }
    }
    let singing = separate(planar)?;
    let bytes: Vec<u8> = (0..length)
      .flat_map(|frame| (0..SIDES).map(move |side| side * CHUNK + frame))
      .flat_map(|index| singing[index].to_le_bytes())
      .collect();
    writer
      .write_all(&bytes)
      .map_err(|error| format!("Could not write the singing: {error}"))?;
    start += length;
    #[expect(clippy::cast_precision_loss, reason = "a fraction")]
    progress(start as f32 / total as f32);
  }
  Ok(())
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
