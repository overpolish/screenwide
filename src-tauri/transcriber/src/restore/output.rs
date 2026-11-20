// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The restored voice as it is written: each stretch in its place, with
//! silence between them where the app left a wait out. It is written in
//! order and only ever added to, so a job stopped part-way leaves what it
//! finished, and taking it up again goes on from there.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::ops::Range;
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
  /// How many whole samples a job before left at `path`.
  pub(super) fn left_at(path: &Path) -> usize {
    std::fs::metadata(path).map_or(0, |metadata| to_usize(metadata.len() / 4))
  }

  /// The output at `path`, kept up to sample `from` and written on from
  /// there.
  pub(super) fn from(path: &Path, from: usize) -> Result<Self, String> {
    let file = OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(false)
      .open(path)
      .map_err(failed)?;
    file.set_len(from as u64 * 4).map_err(failed)?;
    let mut file = BufWriter::new(file);
    std::io::Seek::seek(file.get_mut(), std::io::SeekFrom::End(0)).map_err(failed)?;
    Ok(Self {
      file,
      written: from,
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

  /// `samples`, written where the output has reached, faded in and out at
  /// the ends of the `stretch` they lie in, in output samples.
  pub(super) fn piece(&mut self, samples: &[f32], stretch: &Range<usize>) -> Result<(), String> {
    let start = self.written;
    let faded: Vec<f32> = samples
      .iter()
      .enumerate()
      .map(|(offset, sample)| sample * fade(start + offset, stretch))
      .collect();
    self.write(&faded)?;
    // Each piece reaches the disk whole before the next is started, so a job
    // stopped while restoring loses no more than the piece under way.
    self.file.flush().map_err(failed)
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

#[expect(clippy::cast_precision_loss, reason = "within FADE of an end")]
fn fade(at: usize, stretch: &Range<usize>) -> f32 {
  let (from_start, from_end) = (at - stretch.start, stretch.end - at);
  if from_start >= FADE && from_end > FADE {
    return 1.0;
  }
  let edge = from_start.min(from_end - 1) as f32 + 0.5;
  (edge / FADE as f32).min(1.0)
}

fn to_usize(samples: u64) -> usize {
  usize::try_from(samples).unwrap_or(usize::MAX)
}

fn failed(error: std::io::Error) -> String {
  format!("Could not write the voice: {error}")
}
