// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio sound: a voice rebuilt as a close, dry studio take by Sidon, a
//! speech restoration model from the University of Tokyo. A speech encoder
//! hears what was said and how, past the room and the microphone, and a
//! vocoder speaks it again at 48 kHz.
//!
//! The encoder compares every moment of what it hears with every other, so
//! its memory grows with the square of how much it hears at once. Each
//! stretch the app asks for goes through in chunks of ten seconds, each
//! overlapping the last by one, and is cut over from one chunk to the next
//! inside the overlap (`splice`).

mod fbank;
mod high_pass;
mod output;
mod session;
mod splice;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;
use screenwide_transcriber::{CLEAN_SAMPLE_RATE, SAMPLE_RATE};

use self::fbank::{Fbank, WIDTH};
use self::high_pass::HighPass;
use self::output::{Output, Stretch};
use self::session::session;
use self::splice::splice;

/// Seconds heard at once, and how many of them overlap the chunk before: the
/// run-up a chunk hears before it is listened to, and the stretch the cut
/// over to it is looked for in.
const CHUNK_SECONDS: usize = 10;
const OVERLAP_SECONDS: usize = 1;
/// Output samples for each input sample: 16 kHz in, 48 kHz out.
const UPSAMPLE: usize = (CLEAN_SAMPLE_RATE / SAMPLE_RATE) as usize;
/// Silence put either side of each chunk, as Sidon's own demo does.
const PAD: usize = 160;
/// How far behind the input the restored voice comes out, in output
/// samples, measured against the recording; taken off so the voice stays
/// lined up with the picture.
const DELAY: usize = 310;
/// The rumble below this is taken out first, as the demo does.
const HIGH_PASS_HZ: f64 = 50.0;
/// The loudest input sample is brought here, as the demo does.
const PEAK: f32 = 0.9;
const HIDDEN: usize = 1_024;

struct Models {
  fbank: Fbank,
  encoder: Session,
  vocoder: Session,
}

/// Restores the `stretches` of the mono samples at `audio`, at the
/// protocol's sample rate, into `output` at the cleaning rate, three samples
/// for each one in and lined up with them, with silence between, by the
/// encoder and vocoder at `features` and `decoder`. Tells `progress` how far
/// it has got.
pub fn restore_speech(
  (features, decoder): (&Path, &Path),
  audio: &Path,
  stretches: &[[u64; 2]],
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the audio: {error}");
  let total = std::fs::metadata(audio).map_err(unreadable)?.len() / 4;
  // Sorted, apart and inside the audio, whatever was asked.
  let mut wanted: Vec<(u64, u64)> = Vec::with_capacity(stretches.len());
  for &[start, end] in stretches {
    let start = start.max(wanted.last().map_or(0, |&(_, end)| end));
    let end = end.min(total);
    if end > start {
      wanted.push((start, end));
    }
  }
  let mut out = Output::create(output)?;
  let work = wanted
    .iter()
    .map(|(start, end)| end - start)
    .sum::<u64>()
    .max(1);
  if !wanted.is_empty() {
    let peak = loudest(audio)?;
    let gain = if peak > 0.0 { PEAK / peak } else { 1.0 };
    let mut models = Models {
      fbank: Fbank::new(SAMPLE_RATE),
      encoder: session(features)?,
      vocoder: session(decoder)?,
    };
    let mut reader = BufReader::new(File::open(audio).map_err(unreadable)?);
    let mut done = 0;
    for (start, end) in wanted {
      let (start, length) = (to_usize(start), to_usize(end - start));
      out.silence_until(start * UPSAMPLE)?;
      reader
        .seek(SeekFrom::Start(start as u64 * 4))
        .map_err(unreadable)?;
      let mut stretch = out.stretch(length * UPSAMPLE, DELAY);
      restore_stretch(
        &mut models,
        &mut reader,
        length,
        gain,
        &mut stretch,
        &mut |restored| {
          #[expect(clippy::cast_precision_loss, reason = "a fraction")]
          progress((done + restored as u64) as f32 / work as f32);
        },
      )?;
      stretch.finish()?;
      done += length as u64;
    }
  }
  out.silence_until(to_usize(total) * UPSAMPLE)?;
  out.finish()
}

/// Restores the `total` samples `reader` is at into `writer`, `gain` taking
/// the loudest sample of the whole track to [`PEAK`]. Tells `progress` how
/// many samples are done.
fn restore_stretch(
  models: &mut Models,
  reader: &mut impl Read,
  total: usize,
  gain: f32,
  writer: &mut Stretch<'_>,
  progress: &mut dyn FnMut(usize),
) -> Result<(), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the audio: {error}");
  let mut filter = HighPass::new(f64::from(SAMPLE_RATE), HIGH_PASS_HZ);
  let (chunk, overlap) = (
    CHUNK_SECONDS * SAMPLE_RATE as usize,
    OVERLAP_SECONDS * SAMPLE_RATE as usize,
  );
  let hop = chunk - overlap;
  // What the chunk before made of the overlap, kept up to the cut.
  let mut held: Vec<f32> = Vec::new();
  let mut window: Vec<f32> = Vec::with_capacity(chunk + 2 * PAD);
  let mut start = 0;
  loop {
    // The overlap carried from the chunk before, then what is new.
    let carried = window.len().saturating_sub(PAD * 2).min(overlap);
    let keep: Vec<f32> = if start == 0 {
      Vec::new()
    } else {
      window[PAD + hop..PAD + hop + carried].to_vec()
    };
    window.clear();
    window.resize(PAD, 0.0);
    window.extend_from_slice(&keep);
    let wanted = (chunk - keep.len()).min(total - start - keep.len());
    read_into(reader, wanted, &mut window).map_err(unreadable)?;
    for sample in &mut window[PAD + keep.len()..] {
      *sample = filter.process(*sample * gain);
    }
    let length = window.len() - PAD;
    window.resize(window.len() + PAD, 0.0);
    let last = start + length >= total;
    let mut voice = restore_chunk(models, &window)?;
    voice.resize(length * UPSAMPLE, 0.0);
    splice(&held, &mut voice);
    if last {
      return writer.push(&voice);
    }
    let settled = hop * UPSAMPLE;
    writer.push(&voice[..settled])?;
    held = voice[settled..].to_vec();
    start += hop;
    progress(start);
  }
}

/// The voice the model makes of one padded chunk.
fn restore_chunk(models: &mut Models, samples: &[f32]) -> Result<Vec<f32>, String> {
  let failed = |error: ort::Error| format!("Studio sound could not run: {error}");
  let (features, rows) = models.fbank.features(samples);
  if rows == 0 {
    return Ok(Vec::new());
  }
  let input = Tensor::from_array(([1_usize, rows, WIDTH], features)).map_err(failed)?;
  let outputs = models
    .encoder
    .run(ort::inputs!["input_features" => input])
    .map_err(failed)?;
  let (_, hidden) = outputs["last_hidden_state"]
    .try_extract_tensor::<f32>()
    .map_err(failed)?;
  // The encoder speaks frame by frame; the vocoder listens band by band.
  let mut transposed = vec![0.0_f32; rows * HIDDEN];
  for (row, values) in hidden.as_chunks::<HIDDEN>().0.iter().enumerate() {
    for (band, &value) in values.iter().enumerate() {
      transposed[band * rows + row] = value;
    }
  }
  drop(outputs);
  let hidden = Tensor::from_array(([1_usize, HIDDEN, rows], transposed)).map_err(failed)?;
  let outputs = models
    .vocoder
    .run(ort::inputs!["hidden" => hidden])
    .map_err(failed)?;
  let (_, voice) = outputs["wav"].try_extract_tensor::<f32>().map_err(failed)?;
  Ok(voice.to_vec())
}

fn to_usize(samples: u64) -> usize {
  usize::try_from(samples).unwrap_or(usize::MAX)
}

/// The loudest sample in the file at `audio`.
fn loudest(audio: &Path) -> Result<f32, String> {
  let mut reader = BufReader::new(
    File::open(audio).map_err(|error| format!("Could not read the audio: {error}"))?,
  );
  let mut bytes = vec![0_u8; 1 << 16];
  let mut filled = 0;
  let mut peak = 0.0_f32;
  loop {
    let read = reader
      .read(&mut bytes[filled..])
      .map_err(|error| format!("Could not read the audio: {error}"))?;
    if read == 0 {
      return Ok(peak);
    }
    filled += read;
    let (samples, rest) = bytes[..filled].as_chunks::<4>();
    for sample in samples {
      peak = peak.max(f32::from_le_bytes(*sample).abs());
    }
    // A read can end partway through a sample; its first bytes wait for the
    // rest rather than shifting every sample after it.
    let split = filled - rest.len();
    bytes.copy_within(split..filled, 0);
    filled -= split;
  }
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
