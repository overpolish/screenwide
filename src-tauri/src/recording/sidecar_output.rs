// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a cursor or keyboard sidecar's records go: straight into the
//! recording's JSONL file, or into a bounded window of the recent past that
//! the replay buffer cuts clips from.
//!
//! A clip that starts partway through the window still has to open in the
//! state the screen was in at that moment: the cursor's shape, where it was,
//! whether it was hidden, which buttons and keys were held. Records that fall
//! out of the window are therefore folded into a [`Baseline`] rather than
//! dropped, and a clip restates that state at its own time zero.

// The rolling half is only reached by the macOS replay buffer so far.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::Serialize;

/// A sidecar record that sits on the recording's timeline.
pub(crate) trait TimedRecord: Clone + Serialize {
  /// `None` for records that describe the file rather than a moment in it.
  fn timestamp_us(&self) -> Option<u64>;
  fn at_timestamp_us(&self, timestamp_us: u64) -> Self;
}

/// What earlier records leave behind that a later clip still has to show.
pub(crate) trait Baseline<R>: Clone + Default {
  fn absorb(&mut self, record: &R);
  /// The state as records, in the order a recording would have written them.
  /// Their timestamps are rewritten to the clip's start by the caller.
  fn restate(&self) -> Vec<R>;
}

pub(crate) enum SidecarOutput<R, B> {
  File(BufWriter<File>),
  Rolling(RollingRecords<R, B>),
}

impl<R: TimedRecord, B: Baseline<R>> SidecarOutput<R, B> {
  /// Opens `path` and writes the header line every sidecar file starts with.
  pub(crate) fn file(path: &Path, header: &R) -> Result<Self, String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let mut output = Self::File(BufWriter::new(file));
    output.write(header)?;
    output.flush()?;
    Ok(output)
  }

  pub(crate) fn write(&mut self, record: &R) -> Result<(), String> {
    match self {
      Self::File(writer) => write_line(writer, record),
      Self::Rolling(rolling) => {
        rolling.push(record.clone());
        Ok(())
      }
    }
  }

  pub(crate) fn flush(&mut self) -> Result<(), String> {
    match self {
      Self::File(writer) => writer.flush().map_err(|error| error.to_string()),
      Self::Rolling(_) => Ok(()),
    }
  }

  pub(crate) fn rolling(&self) -> Option<&RollingRecords<R, B>> {
    match self {
      Self::File(_) => None,
      Self::Rolling(rolling) => Some(rolling),
    }
  }
}

/// The records of the last `horizon_us`, plus the state everything older
/// left behind.
pub(crate) struct RollingRecords<R, B> {
  baseline: B,
  horizon_us: u64,
  records: VecDeque<R>,
}

impl<R: TimedRecord, B: Baseline<R>> RollingRecords<R, B> {
  pub(crate) fn new(horizon_us: u64) -> Self {
    Self {
      baseline: B::default(),
      horizon_us,
      records: VecDeque::new(),
    }
  }

  fn push(&mut self, record: R) {
    let newest = record.timestamp_us();
    self.records.push_back(record);
    let Some(cutoff) = newest.and_then(|newest| newest.checked_sub(self.horizon_us)) else {
      return;
    };
    while self
      .records
      .front()
      .is_some_and(|front| front.timestamp_us().is_none_or(|at| at < cutoff))
    {
      let old = self.records.pop_front().expect("checked above");
      self.baseline.absorb(&old);
    }
  }

  /// The records of `start_us..=end_us`, rebased so the clip starts at zero
  /// and opening with the state the screen was already in.
  pub(crate) fn clip(&self, start_us: u64, end_us: u64) -> Vec<R> {
    let mut state = self.baseline.clone();
    let mut inside = Vec::new();
    for record in &self.records {
      match record.timestamp_us() {
        Some(at) if at < start_us => state.absorb(record),
        Some(at) if at <= end_us => inside.push(record.at_timestamp_us(at - start_us)),
        _ => {}
      }
    }
    let mut clip: Vec<R> = state
      .restate()
      .iter()
      .map(|record| record.at_timestamp_us(0))
      .collect();
    clip.extend(inside);
    clip
  }
}

/// Writes a complete sidecar file: `header`, then `records`.
pub(crate) fn write_file<R: Serialize>(
  path: &Path,
  header: &R,
  records: &[R],
) -> Result<(), String> {
  let file = File::create(path).map_err(|error| error.to_string())?;
  let mut writer = BufWriter::new(file);
  write_line(&mut writer, header)?;
  for record in records {
    write_line(&mut writer, record)?;
  }
  writer.flush().map_err(|error| error.to_string())
}

fn write_line<R: Serialize>(writer: &mut BufWriter<File>, record: &R) -> Result<(), String> {
  serde_json::to_writer(&mut *writer, record).map_err(|error| error.to_string())?;
  writer.write_all(b"\n").map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
