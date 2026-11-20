// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How loud a voice is while it speaks, so the restored voice can be brought
//! to the level it was recorded at. The finish's compressor was set by ear
//! with the voice there, and the switch then changes how the voice sounds,
//! not how loud it is.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Each stretch measured, in seconds.
const FRAME_SECONDS: f32 = 0.04;
/// The share of the quietest stretches left out as pauses and breaths.
const QUIET_SHARE: f32 = 0.6;
/// Samples read at a time.
const CHUNK: usize = 1 << 16;

/// The root mean square of the loudest stretches of the mono samples in
/// `path`, at `rate`: the voice while it speaks, leaving its pauses out.
/// Zero for silence.
pub(super) fn voiced(path: &Path, rate: u32) -> Result<f32, String> {
  let unreadable = |error: std::io::Error| format!("Could not read the voice: {error}");
  let frame = ((rate as f32 * FRAME_SECONDS) as usize).max(1);
  let mut reader = BufReader::new(File::open(path).map_err(unreadable)?);
  let mut bytes = vec![0_u8; CHUNK * 4];
  let mut frames = Vec::new();
  let (mut sum, mut count) = (0.0_f64, 0);
  loop {
    let read = read_whole_samples(&mut reader, &mut bytes).map_err(unreadable)?;
    if read == 0 {
      break;
    }
    for sample in bytes[..read].as_chunks::<4>().0 {
      let sample = f64::from(f32::from_le_bytes(*sample));
      sum += sample * sample;
      count += 1;
      if count == frame {
        frames.push(sum / frame as f64);
        (sum, count) = (0.0, 0);
      }
    }
  }
  Ok(loudest_share(&mut frames))
}

/// The root mean square over the frames, given as mean squares, above the
/// [`QUIET_SHARE`] quietest.
fn loudest_share(frames: &mut [f64]) -> f32 {
  if frames.is_empty() {
    return 0.0;
  }
  let from = ((frames.len() as f32 * QUIET_SHARE) as usize).min(frames.len() - 1);
  frames.select_nth_unstable_by(from, f64::total_cmp);
  let loud = &frames[from..];
  (loud.iter().sum::<f64>() / loud.len() as f64).sqrt() as f32
}

/// Fills `bytes` as far as the file goes, a whole number of samples.
fn read_whole_samples(reader: &mut impl Read, bytes: &mut [u8]) -> std::io::Result<usize> {
  let mut filled = 0;
  while filled < bytes.len() {
    match reader.read(&mut bytes[filled..])? {
      0 => break,
      read => filled += read,
    }
  }
  Ok(filled - filled % 4)
}

#[cfg(test)]
mod tests {
  use super::loudest_share;

  #[test]
  fn pauses_do_not_lower_the_level() {
    // Speech at 0.1 for half the time, a pause at 0.001 for the rest.
    let mut speaking: Vec<f64> = (0..300)
      .map(|index| if index % 2 == 0 { 0.01 } else { 1e-6 })
      .collect();
    let level = loudest_share(&mut speaking);
    assert!((level - 0.1).abs() < 1e-4, "{level}");
    let steady = loudest_share(&mut vec![0.01; 300]);
    assert!((steady - 0.1).abs() < 1e-4, "{steady}");
  }
}
