// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Telling sound from silence in mono float audio, by the level of short
//! windows of it.

#[cfg(test)]
mod tests;

/// Long enough to average out single samples, short enough that a word spans
/// several windows.
const WINDOW_MS: u32 = 30;
/// Below anything said into a working microphone: a quiet "Hello?" peaked at
/// about -35 dBFS, while a note recorded with no sound reaching it stayed
/// under -69.
const SILENCE_DBFS: f32 = -50.0;

/// The RMS level of each `WINDOW_MS` window of `samples`, mono at
/// `sample_rate`, in dBFS. A shorter window at the end counts as one.
pub(crate) fn window_levels(samples: &[f32], sample_rate: u32) -> impl Iterator<Item = f32> + '_ {
  let window = (sample_rate * WINDOW_MS / 1_000).max(1) as usize;
  samples.chunks(window).map(|chunk| {
    let power = chunk.iter().map(|sample| sample * sample).sum::<f32>() / chunk.len() as f32;
    10.0 * power.max(f32::MIN_POSITIVE).log10()
  })
}

/// Whether no window of `samples` rises above silence.
pub(crate) fn is_silent(samples: &[f32], sample_rate: u32) -> bool {
  window_levels(samples, sample_rate).all(|level| level <= SILENCE_DBFS)
}
