// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The restored voice brought to the level it was recorded at, with its pauses
//! turned down. The model restores a breath or a click between words as
//! cleanly as the words, so the gate here holds closer to the speech than
//! Reduce noise's does: an in-breath right before a word sits inside Reduce
//! noise's margins, and is still heard.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use screenwide_transcriber::CLEAN_SAMPLE_RATE;

use super::super::analysis::SpeechMap;
use super::super::pause_gate::{Margins, PauseGate};
use crate::runner::samples::Samples;

/// How close to speech the gate holds open, and how long it takes to close.
const BREATH_MARGINS: Margins = Margins {
  before_ms: 60.0,
  after_ms: 100.0,
  fade_ms: 30.0,
};
/// Samples gained and gated at a time.
const CHUNK: usize = 1 << 16;

/// The samples in `restored`, at the cleaning rate, brought from speaking
/// at `restored_level` to `recorded_level` (see `level::voiced`) and, where
/// `speech` hears none, turned down.
pub(super) fn level_and_gate(
  restored: &Path,
  (restored_level, recorded_level): (f32, f32),
  speech: Option<&SpeechMap>,
) -> Result<Samples, String> {
  let unreadable = |error: std::io::Error| format!("Could not read the restored voice: {error}");
  let unwritable = |error: std::io::Error| format!("Could not write the restored voice: {error}");
  let gain = if restored_level > 0.0 && recorded_level > 0.0 {
    recorded_level / restored_level
  } else {
    1.0
  };
  let length = std::fs::metadata(restored).map_err(unreadable)?.len() / 4;
  let mut gate =
    speech.map(|speech| PauseGate::with_margins(speech, CLEAN_SAMPLE_RATE, length, BREATH_MARGINS));
  let mut reader = BufReader::new(File::open(restored).map_err(unreadable)?);
  let gated = Samples::scratch();
  let mut writer = BufWriter::new(File::create(gated.path()).map_err(unwritable)?);
  let mut bytes = vec![0_u8; CHUNK * 4];
  let mut samples = Vec::with_capacity(CHUNK);
  let mut offset = 0;
  while offset < length {
    let count = CHUNK.min(usize::try_from(length - offset).unwrap_or(CHUNK));
    reader
      .read_exact(&mut bytes[..count * 4])
      .map_err(unreadable)?;
    samples.clear();
    samples.extend(
      bytes[..count * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|sample| f32::from_le_bytes(*sample) * gain),
    );
    if let Some(gate) = &mut gate {
      gate.apply(offset, &mut samples);
    }
    for (sample, out) in samples.iter().zip(bytes.as_chunks_mut::<4>().0) {
      *out = sample.to_le_bytes();
    }
    writer.write_all(&bytes[..count * 4]).map_err(unwritable)?;
    offset += count as u64;
  }
  writer.flush().map_err(unwritable)?;
  Ok(gated)
}
