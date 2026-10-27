// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the picture while it is recorded, from the frames the capture
//! already holds, into a file beside the movie (`<movie>-picture.jsonl`).
//! The same on every platform: each capture hands over its frames as they
//! are written and shrinks one to grey only when a sample is due. Nothing is
//! decoded afterwards.
//!
//! A screen that stops changing stops sending frames, on macOS
//! (`SCFrameStatus.idle`) as through the composition on Windows, so a slot
//! with no new frame is the last frame again: changed nothing, which is what
//! the screen did. Those slots are filled in when the next frame, or the
//! end, comes.

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::measure::{Measure, PictureReading, SAMPLE_SIZE};
use crate::sample::SAMPLES_PER_SECOND;

const SLOT_NS: i64 = 1_000_000_000 / SAMPLES_PER_SECOND as i64;
const VERSION: u32 = 1;

/// Where the readings of the movie at `movie` are kept.
pub fn readings_path(movie: &Path) -> PathBuf {
  let stem = movie
    .file_stem()
    .map_or_else(Default::default, |stem| stem.to_string_lossy());
  movie.with_file_name(format!("{stem}-picture.jsonl"))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
enum Line {
  Header {
    height: usize,
    samples_per_second: u32,
    version: u32,
    width: usize,
  },
  Reading {
    changed: f32,
    ms: u64,
    novelty: f32,
  },
}

pub struct PictureTap {
  file: BufWriter<File>,
  last: Option<Vec<u8>>,
  measure: Measure,
  next_slot: i64,
}

impl PictureTap {
  /// Starts the readings of the movie being written at `movie`.
  pub fn create(movie: &Path) -> Result<Self, String> {
    let file = File::create(readings_path(movie)).map_err(|error| error.to_string())?;
    let mut tap = Self {
      file: BufWriter::new(file),
      last: None,
      measure: Measure::new(f64::from(SAMPLES_PER_SECOND)),
      next_slot: 0,
    };
    tap.write(&Line::Header {
      height: SAMPLE_SIZE.1,
      samples_per_second: SAMPLES_PER_SECOND,
      version: VERSION,
      width: SAMPLE_SIZE.0,
    })?;
    Ok(tap)
  }

  /// A frame written at `pts_ns` into the movie. `grey` gives its picture
  /// at `SAMPLE_SIZE` and is asked only when this frame is the first since
  /// a sample fell due, so most frames cost nothing.
  pub fn frame(
    &mut self,
    pts_ns: i64,
    grey: impl FnOnce() -> Option<Vec<u8>>,
  ) -> Result<(), String> {
    if pts_ns < self.next_slot * SLOT_NS {
      return Ok(());
    }
    let Some(frame) = grey() else {
      return Ok(());
    };
    let slot = pts_ns / SLOT_NS;
    self.repeat_until(slot)?;
    self.measure_slot(frame.clone())?;
    self.next_slot = slot + 1;
    self.last = Some(frame);
    Ok(())
  }

  /// Fills the readings to the movie's end, at `end_ns`, and closes them.
  pub fn finish(mut self, end_ns: i64) -> Result<(), String> {
    self.repeat_until(end_ns / SLOT_NS)?;
    self.file.flush().map_err(|error| error.to_string())
  }

  /// The slots before `slot` that saw no new frame: the last one again.
  fn repeat_until(&mut self, slot: i64) -> Result<(), String> {
    while self.next_slot < slot {
      if let Some(last) = self.last.clone() {
        self.measure_slot(last)?;
      }
      self.next_slot += 1;
    }
    Ok(())
  }

  fn measure_slot(&mut self, frame: Vec<u8>) -> Result<(), String> {
    let reading = self.measure.next(frame);
    self.write(&Line::Reading {
      changed: reading.changed,
      ms: reading.ms,
      novelty: reading.novelty,
    })
  }

  fn write(&mut self, line: &Line) -> Result<(), String> {
    serde_json::to_writer(&mut self.file, line).map_err(|error| error.to_string())?;
    self
      .file
      .write_all(b"\n")
      .map_err(|error| error.to_string())
  }
}

/// The readings written beside the movie at `movie`.
pub fn read_readings(movie: &Path) -> Result<Vec<PictureReading>, String> {
  let file = File::open(readings_path(movie)).map_err(|error| error.to_string())?;
  let mut readings = Vec::new();
  for line in BufReader::new(file).lines() {
    let line = line.map_err(|error| error.to_string())?;
    match serde_json::from_str::<Line>(&line).map_err(|error| error.to_string())? {
      Line::Header { version, .. } if version != VERSION => {
        return Err(format!("These readings are version {version}"));
      }
      Line::Header { .. } => {}
      Line::Reading {
        changed,
        ms,
        novelty,
      } => readings.push(PictureReading {
        changed,
        ms,
        novelty,
      }),
    }
  }
  Ok(readings)
}
