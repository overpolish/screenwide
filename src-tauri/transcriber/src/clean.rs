// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A voice with everything else taken out, by DeepFilterNet 3: a speech
//! enhancer that learnt what a voice is, so it takes out passing clicks and
//! rustles as well as steady hiss and hum, where subtracting a room's noise
//! takes out only the steady part.
//!
//! The model runs on one core, so the track is cut into parts cleaned side
//! by side. The model carries what it has heard from one moment to the next,
//! so each part starts listening a little before its own stretch, and parts
//! that meet are blended over a few milliseconds where both were cleaned.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::Array2;
use screenwide_transcriber::CLEAN_SAMPLE_RATE;

/// How long each part is: a minute. Short enough that every core stays busy
/// to the end, long enough that the lead-in is little extra work.
const PART: usize = 2_880_000;
/// How long a part listens before its own stretch: 10 s. The model's sense of
/// how loud the voice is settles over seconds of speech, not of time; against
/// one pass over a whole recording, 2 s left some first words after a join
/// up to 12 dB quieter, and 10 s brought them within a fraction of a dB.
const LEAD_IN: usize = 480_000;
/// Where parts meet, both are cleaned for 20 ms and blended.
const BLEND: usize = 960;
/// Blocks of 10 ms that the cleaned voice's loudness is compared in.
const BLOCK: usize = 480;
/// A cleaned block quieter than this, in full-scale RMS (-50 dBFS), holds no
/// voice and says nothing about how much quieter the voice came out.
const VOICE_RMS: f32 = 0.003_2;
/// The most the cleaned voice is lifted by, 18 dB.
const MOST_GAIN: f32 = 8.0;
/// Where the lifted voice's loudest sample may reach.
const HEADROOM: f32 = 0.98;
/// How often progress is told, at most.
const PROGRESS_EVERY: Duration = Duration::from_millis(100);

/// The voice's loudness before and after, over the blocks that hold voice.
#[derive(Default)]
struct Loudness {
  voice_in: f64,
  voice_out: f64,
  peak: f32,
  block: usize,
  block_in: f64,
  block_out: f64,
}

impl Loudness {
  fn add(&mut self, recorded: f32, cleaned: f32) {
    self.block_in += f64::from(recorded * recorded);
    self.block_out += f64::from(cleaned * cleaned);
    self.peak = self.peak.max(cleaned.abs());
    self.block += 1;
    if self.block == BLOCK {
      if self.block_out / BLOCK as f64 > f64::from(VOICE_RMS * VOICE_RMS) {
        self.voice_in += self.block_in;
        self.voice_out += self.block_out;
      }
      (self.block, self.block_in, self.block_out) = (0, 0.0, 0.0);
    }
  }

  fn merge(&mut self, other: &Self) {
    self.voice_in += other.voice_in;
    self.voice_out += other.voice_out;
    self.peak = self.peak.max(other.peak);
  }

  /// The gain that puts the voice back at its recorded loudness, short of
  /// clipping, and never below one.
  fn gain(&self) -> f32 {
    let gain = if self.voice_out > 0.0 {
      ((self.voice_in / self.voice_out).sqrt() as f32).clamp(1.0, MOST_GAIN)
    } else {
      1.0
    };
    gain
      .min(HEADROOM / self.peak.max(f32::MIN_POSITIVE))
      .max(1.0)
  }
}

/// What a part leaves for the parts beside it to be blended with.
struct Cleaned {
  loudness: Loudness,
  /// Its first [`BLEND`] samples, blended with the part before.
  head: Vec<f32>,
  /// The [`BLEND`] samples after its end, blended with the part after.
  tail: Vec<f32>,
}

/// Cleans the mono samples at `audio` into `output`, as many of them and
/// lined up with them, telling `progress` how far it has got.
///
/// The enhancer takes the room's echo off a voice along with the noise,
/// which can leave it several dB quieter than it was recorded. Answers with
/// the gain that puts the voice back at its recorded loudness.
pub fn clean_speech(
  audio: &Path,
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<f32, String> {
  let total = usize::try_from(
    std::fs::metadata(audio)
      .map_err(|error| format!("Could not read the audio: {error}"))?
      .len()
      / 4,
  )
  .unwrap_or(usize::MAX);
  File::create(output)
    .and_then(|file| file.set_len(total as u64 * 4))
    .map_err(|error| format!("Could not write the voice: {error}"))?;
  let parts: Vec<(usize, usize)> = (0..total.div_ceil(PART).max(1))
    .map(|index| (index * PART, ((index + 1) * PART).min(total)))
    .collect();
  let last = parts.len() - 1;
  let work: usize = parts
    .iter()
    .enumerate()
    .map(|(index, &(start, end))| feed_range(index, last, start, end, total).len())
    .sum();
  let params = DfParams::default();
  let next = AtomicUsize::new(0);
  let done = AtomicUsize::new(0);
  let cleaned: Mutex<Vec<Option<Result<Cleaned, String>>>> =
    Mutex::new((0..parts.len()).map(|_| None).collect());
  let workers = std::thread::available_parallelism()
    .map_or(1, |count| count.get())
    .min(parts.len());
  std::thread::scope(|scope| {
    let handles: Vec<_> = (0..workers)
      .map(|_| {
        scope.spawn(|| {
          // One model a worker, carried from part to part: what it heard of
          // the last part is long gone by the end of the next one's lead-in.
          let mut model = DfTract::new(params.clone(), &RuntimeParams::default_with_ch(1))
            .map_err(|error| format!("Could not load the speech enhancer: {error}"));
          loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(&(start, end)) = parts.get(index) else {
              break;
            };
            let result = match &mut model {
              Ok(model) => clean_part(
                audio,
                output,
                model,
                (index, last),
                (start, end, total),
                &done,
              ),
              Err(error) => Err(error.clone()),
            };
            cleaned
              .lock()
              .unwrap_or_else(|poisoned| poisoned.into_inner())[index] = Some(result);
          }
        })
      })
      .collect();
    while !handles.iter().all(|handle| handle.is_finished()) {
      progress(done.load(Ordering::Relaxed) as f32 / work.max(1) as f32);
      std::thread::sleep(PROGRESS_EVERY);
    }
  });
  let cleaned = cleaned
    .into_inner()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .into_iter()
    .map(|part| part.unwrap_or_else(|| Err("A part of the voice was not cleaned".to_owned())))
    .collect::<Result<Vec<_>, _>>()?;

  let mut loudness = Loudness::default();
  let mut file = OpenOptions::new()
    .write(true)
    .open(output)
    .map_err(|error| format!("Could not write the voice: {error}"))?;
  for (index, part) in cleaned.iter().enumerate() {
    loudness.merge(&part.loudness);
    let Some(after) = cleaned.get(index + 1) else {
      continue;
    };
    // The part after fades in as this one fades out.
    let length = part.tail.len().min(after.head.len());
    let blended: Vec<u8> = (0..length)
      .flat_map(|sample| {
        let weight = (sample as f32 + 0.5) / length as f32;
        let value = part.tail[sample] * (1.0 - weight) + after.head[sample] * weight;
        loudness.peak = loudness.peak.max(value.abs());
        value.to_ne_bytes()
      })
      .collect();
    file
      .seek(SeekFrom::Start(parts[index].1 as u64 * 4))
      .and_then(|_| file.write_all(&blended))
      .map_err(|error| format!("Could not write the voice: {error}"))?;
  }
  progress(1.0);
  Ok(loudness.gain())
}

/// The samples the `index`th of `last + 1` parts reads: its own stretch, from
/// a lead-in before it unless it is the first, to a blend after it unless it
/// is the last.
fn feed_range(
  index: usize,
  last: usize,
  start: usize,
  end: usize,
  total: usize,
) -> std::ops::Range<usize> {
  let from = if index == 0 {
    start
  } else {
    start.saturating_sub(LEAD_IN)
  };
  let to = if index == last {
    end
  } else {
    (end + BLEND).min(total)
  };
  from..to
}

/// Cleans the samples from `start` to `end` into the same place in `output`,
/// keeping the stretches it shares with the parts beside it to be blended.
fn clean_part(
  audio: &Path,
  output: &Path,
  model: &mut DfTract,
  (index, last): (usize, usize),
  (start, end, total): (usize, usize, usize),
  done: &AtomicUsize,
) -> Result<Cleaned, String> {
  if model.sr != CLEAN_SAMPLE_RATE as usize {
    return Err("The speech enhancer works at another rate".to_owned());
  }
  let feed = feed_range(index, last, start, end, total);
  let written_from = if index > 0 {
    (start + BLEND).min(end)
  } else {
    start
  };
  let mut input =
    File::open(audio).map_err(|error| format!("Could not read the audio: {error}"))?;
  input
    .seek(SeekFrom::Start(feed.start as u64 * 4))
    .map_err(|error| format!("Could not read the audio: {error}"))?;
  let mut input = BufReader::new(input).take((feed.len() * 4) as u64);
  let mut writer = OpenOptions::new()
    .write(true)
    .open(output)
    .and_then(|mut file| {
      file
        .seek(SeekFrom::Start(written_from as u64 * 4))
        .map(|_| file)
    })
    .map(BufWriter::new)
    .map_err(|error| format!("Could not write the voice: {error}"))?;

  // What comes out lags what goes in by the transform's overlap and the
  // frames the model looks ahead.
  let delay = model.fft_size - model.hop_size + model.lookahead * model.hop_size;
  let hop = model.hop_size;
  let mut noisy = Array2::<f32>::zeros((1, hop));
  let mut enhanced = Array2::<f32>::zeros((1, hop));
  let mut bytes = vec![0_u8; hop * 4];
  // The samples in, held until the cleaned sample for the same moment comes
  // out, so the two are measured side by side.
  let mut waiting = std::collections::VecDeque::with_capacity(delay + hop);
  let mut part = Cleaned {
    loudness: Loudness::default(),
    head: Vec::with_capacity(BLEND),
    tail: Vec::with_capacity(BLEND),
  };
  let (mut skipped, mut position) = (0, feed.start);
  while position < feed.end {
    // Past the end the model is fed silence, to let out what it holds.
    let read = read_up_to(&mut input, &mut bytes)?;
    bytes[read..].fill(0);
    for (sample, chunk) in noisy.iter_mut().zip(bytes.as_chunks::<4>().0) {
      *sample = f32::from_ne_bytes(*chunk);
      waiting.push_back(*sample);
    }
    model
      .process(noisy.view(), enhanced.view_mut())
      .map_err(|error| format!("Could not clean the voice: {error}"))?;
    let before = position;
    for &sample in &enhanced {
      // The first outputs of a part are what the model still held.
      if skipped < delay {
        skipped += 1;
        continue;
      }
      if position >= feed.end {
        break;
      }
      let recorded = waiting.pop_front().unwrap_or(0.0);
      if position >= end {
        part.tail.push(sample);
      } else if position >= written_from {
        writer
          .write_all(&sample.to_ne_bytes())
          .map_err(|error| format!("Could not write the voice: {error}"))?;
        part.loudness.add(recorded, sample);
      } else if position >= start {
        part.head.push(sample);
      }
      position += 1;
    }
    done.fetch_add(position - before, Ordering::Relaxed);
  }
  writer
    .flush()
    .map_err(|error| format!("Could not write the voice: {error}"))?;
  Ok(part)
}

/// Fills as much of `buffer` as the file has left, in bytes.
fn read_up_to(input: &mut impl Read, buffer: &mut [u8]) -> Result<usize, String> {
  let mut filled = 0;
  while filled < buffer.len() {
    match input.read(&mut buffer[filled..]) {
      Ok(0) => break,
      Ok(read) => filled += read,
      Err(error) => return Err(format!("Could not read the audio: {error}")),
    }
  }
  Ok(filled)
}
