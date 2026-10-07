// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A voice note's sound: gathered as mono 16-bit samples while the key is
//! held, kept as a WAV file, and read back as a level envelope for the
//! editor to draw.

use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

/// The longest a note keeps, so a key left held cannot fill the memory.
const MAX_NOTE_SECONDS: u32 = 600;

#[derive(Default)]
pub(super) struct NoteAudio {
  pub end: Option<Instant>,
  /// The microphone could not be opened, so there is no note to keep.
  pub failed: bool,
  pub sample_rate: u32,
  pub samples: Vec<i16>,
}

impl NoteAudio {
  /// Takes interleaved `samples` captured at `captured_at`, folded to mono.
  /// Anything captured after the key came up is not the note's.
  pub(super) fn take(&mut self, samples: &[f32], channels: u16, sample_rate: u32, at: Instant) {
    if self.end.is_some_and(|end| at > end) || channels == 0 {
      return;
    }
    if self.sample_rate == 0 {
      self.sample_rate = sample_rate;
    }
    let limit = (MAX_NOTE_SECONDS * self.sample_rate) as usize;
    let channels = usize::from(channels);
    for frame in samples.chunks_exact(channels) {
      if self.samples.len() >= limit {
        return;
      }
      let mono = frame.iter().sum::<f32>() / channels as f32;
      self
        .samples
        .push((mono.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16);
    }
  }

  pub(super) fn duration_ms(&self) -> u64 {
    if self.sample_rate == 0 {
      return 0;
    }
    self.samples.len() as u64 * 1_000 / u64::from(self.sample_rate)
  }
}

/// Writes `samples`, mono at `sample_rate`, as a 16-bit PCM WAV file.
pub(super) fn write_wav(path: &Path, sample_rate: u32, samples: &[i16]) -> Result<(), String> {
  let data_len = u32::try_from(samples.len() * 2).map_err(|_| "The note is too long".to_owned())?;
  let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
  bytes.extend_from_slice(b"RIFF");
  bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
  bytes.extend_from_slice(b"WAVEfmt ");
  bytes.extend_from_slice(&16u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&sample_rate.to_le_bytes());
  bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
  bytes.extend_from_slice(&2u16.to_le_bytes());
  bytes.extend_from_slice(&16u16.to_le_bytes());
  bytes.extend_from_slice(b"data");
  bytes.extend_from_slice(&data_len.to_le_bytes());
  for sample in samples {
    bytes.extend_from_slice(&sample.to_le_bytes());
  }
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let mut file = std::fs::File::create(path).map_err(|error| error.to_string())?;
  file.write_all(&bytes).map_err(|error| error.to_string())
}

/// The note at `path` as `points` peak levels from 0 to 1, for drawing. Reads
/// only the WAV files `write_wav` writes.
pub(crate) fn waveform(path: &Path, points: usize) -> Result<Vec<f32>, String> {
  let mut bytes = Vec::new();
  std::fs::File::open(path)
    .and_then(|mut file| file.read_to_end(&mut bytes))
    .map_err(|error| error.to_string())?;
  let data = bytes
    .get(44..)
    .filter(|_| bytes.starts_with(b"RIFF") && bytes.get(36..40) == Some(b"data"))
    .ok_or_else(|| "The voice note is not a file this version wrote".to_owned())?;
  let samples: Vec<f32> = data
    .as_chunks::<2>()
    .0
    .iter()
    .map(|pair| f32::from(i16::from_le_bytes(*pair)).abs() / f32::from(i16::MAX))
    .collect();
  if samples.is_empty() || points == 0 {
    return Ok(Vec::new());
  }
  let per_point = samples.len().div_ceil(points);
  Ok(
    samples
      .chunks(per_point)
      .map(|chunk| chunk.iter().copied().fold(0.0, f32::max))
      .collect(),
  )
}
