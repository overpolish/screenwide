// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stretches of a recording worth rebuilding, with its long quiet waits
//! left out: a minute spent waiting for something to load is a minute the
//! model would otherwise rebuild into silence. A wait must be both unlikely
//! speech and near the room's quiet, because the voice detector can be most
//! of a second late for a soft first word, and half a second is kept either
//! side of it.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;
use std::path::Path;

use screenwide_transcriber::SAMPLE_RATE;

use super::super::analysis::SpeechMap;

/// Samples in each 10 ms block the track is judged in.
const BLOCK: usize = (SAMPLE_RATE / 100) as usize;
/// A block less likely to be speech than this may be part of a wait...
const UNLIKELY: f32 = 0.1;
/// ...if it is within this of the room's quiet, in dB...
const ABOVE_QUIET_DB: f32 = 12.0;
/// ...or quieter than this anyway, in dBFS.
const SILENT_DB: f32 = -70.0;
/// The room's quiet: the level a tenth of the track's blocks are under.
const QUIET_SHARE: usize = 10;
/// Blocks kept either side of anything heard: half a second.
const KEEP_BLOCKS: usize = 50;
/// The shortest wait left out, after what is kept: two seconds. A shorter
/// one saves too little to be worth a stretch of its own.
const SHORTEST_WAIT_BLOCKS: usize = 200;

/// The stretches of the track in `recorded`, mono samples at
/// [`SAMPLE_RATE`], to rebuild, as `[start, end)` in samples.
pub(super) fn stretches(speech: &SpeechMap, recorded: &Path) -> Result<Vec<[u64; 2]>, String> {
  let (levels, total) = levels(recorded)?;
  let chances = chances(speech, levels.len());
  Ok(
    heard(&levels, &chances)
      .into_iter()
      .map(|blocks| {
        let end = (blocks.end * BLOCK).min(total);
        [(blocks.start * BLOCK) as u64, end as u64]
      })
      .collect(),
  )
}

/// The blocks to rebuild, in order and apart, from each block's level in
/// dBFS and chance of speech.
fn heard(levels: &[f32], chances: &[f32]) -> Vec<Range<usize>> {
  let Some(quiet) = quiet_level(levels) else {
    return Vec::new();
  };
  let loud = |level: f32| level > quiet + ABOVE_QUIET_DB && level > SILENT_DB;
  let busy: Vec<bool> = levels
    .iter()
    .zip(chances)
    .map(|(&level, &chance)| chance >= UNLIKELY || loud(level))
    .collect();
  let mut kept = vec![false; busy.len()];
  for (at, _) in busy.iter().enumerate().filter(|(_, &busy)| busy) {
    let end = (at + KEEP_BLOCKS + 1).min(kept.len());
    kept[at.saturating_sub(KEEP_BLOCKS)..end].fill(true);
  }
  let mut stretches: Vec<Range<usize>> = Vec::new();
  for run in runs(&kept) {
    match stretches.last_mut() {
      Some(last) if run.start - last.end < SHORTEST_WAIT_BLOCKS => last.end = run.end,
      _ => stretches.push(run),
    }
  }
  // Waits at either end are left out only when long enough too.
  if let Some(first) = stretches.first_mut() {
    if first.start < SHORTEST_WAIT_BLOCKS {
      first.start = 0;
    }
  }
  if let Some(last) = stretches.last_mut() {
    if kept.len() - last.end < SHORTEST_WAIT_BLOCKS {
      last.end = kept.len();
    }
  }
  stretches
}

/// The runs of `true` in `flags`.
fn runs(flags: &[bool]) -> Vec<Range<usize>> {
  let mut runs = Vec::new();
  let mut start = None;
  for (at, &flag) in flags.iter().enumerate() {
    match (flag, start) {
      (true, None) => start = Some(at),
      (false, Some(from)) => {
        runs.push(from..at);
        start = None;
      }
      _ => {}
    }
  }
  if let Some(from) = start {
    runs.push(from..flags.len());
  }
  runs
}

fn quiet_level(levels: &[f32]) -> Option<f32> {
  let mut sorted = levels.to_vec();
  sorted.sort_by(f32::total_cmp);
  sorted.get(sorted.len() / QUIET_SHARE).copied()
}

/// Each block's chance of speech: the highest of the windows it touches. A
/// block past the end of the map counts as speech, so it is kept.
fn chances(speech: &SpeechMap, blocks: usize) -> Vec<f32> {
  let block_ms = 1_000.0 * BLOCK as f64 / f64::from(SAMPLE_RATE);
  (0..blocks)
    .map(|block| {
      let first = (block as f64 * block_ms / speech.window_ms) as usize;
      let last = (((block + 1) as f64 * block_ms) / speech.window_ms).ceil() as usize;
      let windows = speech
        .probabilities
        .get(first..last.min(speech.probabilities.len()));
      match windows {
        Some(windows) if !windows.is_empty() => windows.iter().copied().fold(0.0, f32::max),
        _ => 1.0,
      }
    })
    .collect()
}

/// Each block's level in dBFS, and how many samples there are.
fn levels(recorded: &Path) -> Result<(Vec<f32>, usize), String> {
  let unreadable = |error: std::io::Error| format!("Could not read the recording: {error}");
  let mut reader = BufReader::new(File::open(recorded).map_err(unreadable)?);
  let mut bytes = vec![0_u8; BLOCK * 4];
  let (mut levels, mut total) = (Vec::new(), 0);
  loop {
    let mut filled = 0;
    while filled < bytes.len() {
      match reader.read(&mut bytes[filled..]).map_err(unreadable)? {
        0 => break,
        read => filled += read,
      }
    }
    let samples = bytes[..filled].as_chunks::<4>().0;
    if samples.is_empty() {
      return Ok((levels, total));
    }
    total += samples.len();
    let energy = samples
      .iter()
      .map(|sample| f64::from(f32::from_le_bytes(*sample)).powi(2))
      .sum::<f64>()
      / samples.len() as f64;
    levels.push((10.0 * (energy + 1e-12).log10()) as f32);
  }
}
