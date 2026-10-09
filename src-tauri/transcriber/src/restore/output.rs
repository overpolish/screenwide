// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The restored voice as it is written: each stretch in its place, with
//! silence between them where the app left a wait out.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Output samples a stretch fades in and out over, 10 ms at 48 kHz, so it
/// starts and ends without a click. Its ends lie in the quiet the app keeps
/// either side of speech.
const FADE: usize = 480;

pub(super) struct Output {
  file: BufWriter<File>,
  written: usize,
}

impl Output {
  pub(super) fn create(path: &Path) -> Result<Self, String> {
    Ok(Self {
      file: BufWriter::new(File::create(path).map_err(failed)?),
      written: 0,
    })
  }

  /// Silence up to output sample `until`.
  pub(super) fn silence_until(&mut self, until: usize) -> Result<(), String> {
    let silence = [0.0_f32; 4_096];
    while self.written < until {
      let count = (until - self.written).min(silence.len());
      self.write(&silence[..count])?;
    }
    Ok(())
  }

  /// A stretch `length` samples long, whose voice comes `delay` samples
  /// behind what it was made from.
  pub(super) fn stretch(&mut self, length: usize, delay: usize) -> Stretch<'_> {
    Stretch {
      output: self,
      skip: delay,
      at: 0,
      length,
    }
  }

  pub(super) fn finish(mut self) -> Result<(), String> {
    self.file.flush().map_err(failed)
  }

  fn write(&mut self, samples: &[f32]) -> Result<(), String> {
    let mut bytes = Vec::with_capacity(samples.len() * 4);
    for sample in samples {
      bytes.extend_from_slice(&sample.to_le_bytes());
    }
    self.written += samples.len();
    self.file.write_all(&bytes).map_err(failed)
  }
}

/// One stretch of restored voice, with the vocoder's delay taken off the
/// front and as long as the stretch.
pub(super) struct Stretch<'a> {
  output: &'a mut Output,
  skip: usize,
  at: usize,
  length: usize,
}

impl Stretch<'_> {
  pub(super) fn push(&mut self, samples: &[f32]) -> Result<(), String> {
    let skipped = self.skip.min(samples.len());
    self.skip -= skipped;
    let samples = &samples[skipped..];
    let taken = samples.len().min(self.length - self.at);
    let faded: Vec<f32> = samples[..taken]
      .iter()
      .enumerate()
      .map(|(offset, sample)| sample * self.fade(self.at + offset))
      .collect();
    self.at += taken;
    self.output.write(&faded)
  }

  /// Pads the end with silence where the vocoder's delay left it short.
  pub(super) fn finish(mut self) -> Result<(), String> {
    let silence = vec![0.0_f32; self.length - self.at];
    self.push(&silence)
  }

  #[expect(clippy::cast_precision_loss, reason = "within FADE of an end")]
  fn fade(&self, at: usize) -> f32 {
    let from_end = self.length - at;
    if at >= FADE && from_end > FADE {
      return 1.0;
    }
    let edge = at.min(from_end - 1) as f32 + 0.5;
    (edge / FADE as f32).min(1.0)
  }
}

fn failed(error: std::io::Error) -> String {
  format!("Could not write the voice: {error}")
}
