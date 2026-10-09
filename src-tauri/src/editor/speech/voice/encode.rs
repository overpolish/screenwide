// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cleanup run over a whole track. It reads the track once to learn its
//! level: how loud it sounds as recorded and once its tone is set, so the
//! loudness the tone takes away can be given back and the switch changes how
//! the voice sounds rather than how loud it is. The second read cleans it
//! for good, a few seconds at a time, so an hour of audio is never held in
//! memory at once.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Stdio;

use screenwide_transcriber::CLEAN_SAMPLE_RATE as RATE;

use super::biquad::{Loudness, Tone};
use super::declick::Declicker;
use super::deess::Deesser;
use super::deplosive::Deplosive;
use super::level::{self, Frames};
use super::tick::Ticks;
use crate::transcription::runner::Samples;

/// Samples cleaned at a time: about five seconds. Like the context, a
/// multiple of every spectral stage's frame, so each frame falls in the same
/// place whichever run it is in.
const CHUNK: usize = 1 << 18;
/// Samples read either side of each chunk, so a tick, a click, a plosive or
/// a sibilant at its edge is heard whole with what is around it: 340 ms.
const CONTEXT: usize = 1 << 14;
/// Samples read at a time while measuring.
const MEASURE_CHUNK: usize = 1 << 16;
/// How much of the progress bar the measuring read takes.
const MEASURE_SHARE: f32 = 0.1;
/// Most the loudness is given back by after cleaning, in dB.
const MOST_MAKEUP_DB: f32 = 12.0;
/// Peaks held below -1 dB, so the cleaned voice never clips. The lookahead
/// is made up for, so the voice stays lined up with the picture.
const LIMITER: &str = "alimiter=limit=-1dB:attack=5:release=50:level=0:latency=1";

/// Cleans the samples in `input` into a FLAC file at `output`.
pub(super) fn encode(
  input: &Samples,
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let mut reader = Reader::open(input.path())?;
  let measured = reader.measure(&mut |fraction| progress(fraction * MEASURE_SHARE))?;
  let speaking = level::speaking(&measured.shaped);
  let speech = speaking
    .as_ref()
    .map(|speaking| level::speech_energy(&measured.shaped, speaking));
  let makeup_db = speaking.as_ref().map_or(0.0, |speaking| {
    let before = level::speech_energy(&measured.recorded, speaking);
    let after = level::speech_energy(&measured.shaped_heard, speaking);
    (10.0 * (before / after.max(f32::MIN_POSITIVE)).log10()).clamp(0.0, MOST_MAKEUP_DB)
  });
  reader.clean(speech, makeup_db, output, &mut |fraction| {
    progress(MEASURE_SHARE + fraction * (1.0 - MEASURE_SHARE));
  })
}

struct Reader {
  file: File,
  length: usize,
  bytes: Vec<u8>,
  samples: Vec<f32>,
}

impl Reader {
  fn open(path: &Path) -> Result<Self, String> {
    let file =
      File::open(path).map_err(|error| format!("Could not read the microphone: {error}"))?;
    let length = file
      .metadata()
      .map_err(|error| format!("Could not read the microphone: {error}"))?
      .len() as usize
      / 4;
    Ok(Self {
      file,
      length,
      bytes: Vec::new(),
      samples: Vec::new(),
    })
  }

  /// Reads samples `from..to` into [`Self::samples`].
  fn read(&mut self, from: usize, to: usize) -> Result<(), String> {
    self.bytes.resize((to - from) * 4, 0);
    self
      .file
      .seek(SeekFrom::Start(from as u64 * 4))
      .and_then(|_| self.file.read_exact(&mut self.bytes))
      .map_err(|error| format!("Could not read the microphone: {error}"))?;
    self.samples.clear();
    self.samples.extend(
      self
        .bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes)),
    );
    Ok(())
  }

  /// Measures the track as recorded and with its tone set.
  fn measure(&mut self, progress: &mut dyn FnMut(f32)) -> Result<Measured, String> {
    let mut tone = Tone::new(RATE);
    let (mut recorded_loudness, mut shaped_loudness) = (Loudness::new(), Loudness::new());
    let mut weighed = Vec::with_capacity(MEASURE_CHUNK);
    let mut recorded = Frames::new(RATE);
    let (mut shaped, mut shaped_heard) = (Frames::new(RATE), Frames::new(RATE));
    for start in (0..self.length).step_by(MEASURE_CHUNK) {
      self.read(start, (start + MEASURE_CHUNK).min(self.length))?;
      recorded_loudness.weigh(&self.samples, &mut weighed);
      recorded.add(&weighed);
      tone.apply(&mut self.samples);
      shaped.add(&self.samples);
      shaped_loudness.weigh(&self.samples, &mut weighed);
      shaped_heard.add(&weighed);
      progress(start as f32 / self.length as f32);
    }
    Ok(Measured {
      recorded: recorded.energies(),
      shaped: shaped.energies(),
      shaped_heard: shaped_heard.energies(),
    })
  }

  /// Cleans the track into `output` through FFmpeg, for a voice whose speech
  /// has an average energy of `speech`, giving back `makeup_db` after its
  /// tone is set. A silent track has its ticks and clicks taken out and is
  /// shaped, with nothing to de-ess.
  fn clean(
    &mut self,
    speech: Option<f32>,
    makeup_db: f32,
    output: &Path,
    progress: &mut dyn FnMut(f32),
  ) -> Result<(), String> {
    let mut ticks = Ticks::new(RATE);
    let mut declicker = Declicker::new(RATE);
    let mut deplosive = Deplosive::new(RATE);
    let mut deesser = speech.map(|speech| Deesser::new(RATE, speech));
    let mut tone = Tone::new(RATE);
    let (chunk, context) = (CHUNK, CONTEXT);
    let mut child = crate::editor::ffmpeg_command()
      .args(["-nostdin", "-loglevel", "error", "-y", "-f", "f32le", "-ar"])
      .arg(RATE.to_string())
      .args(["-ac", "1", "-i", "pipe:0", "-af"])
      .arg(format!("volume={makeup_db:.2}dB,{LIMITER}"))
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
    let mut bytes = Vec::with_capacity(chunk * 4);
    // A write that fails means FFmpeg stopped, and what it says about why is
    // read below, so the error here adds nothing.
    let mut written = Ok(());
    for start in (0..self.length).step_by(chunk) {
      let end = (start + chunk).min(self.length);
      let from = start.saturating_sub(context);
      self.read(from, (end + context).min(self.length))?;
      ticks.apply(&mut self.samples);
      declicker.apply(&mut self.samples);
      deplosive.apply(&mut self.samples);
      if let Some(deesser) = &mut deesser {
        deesser.apply(&mut self.samples);
      }
      let kept = &mut self.samples[start - from..end - from];
      tone.apply(kept);
      bytes.clear();
      bytes.extend(kept.iter().flat_map(|sample| sample.to_le_bytes()));
      written = input.write_all(&bytes);
      if written.is_err() {
        break;
      }
      progress(end as f32 / self.length as f32);
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
}

/// The energy of each frame of the track.
struct Measured {
  /// Weighted as the ear hears loudness, as recorded.
  recorded: Vec<f32>,
  /// With its tone set.
  shaped: Vec<f32>,
  /// With its tone set, weighted as the ear hears loudness.
  shaped_heard: Vec<f32>,
}
