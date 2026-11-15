// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the person on camera moved more than they do while talking: a hand
//! raised, a lean towards the camera, a turn away.
//!
//! A camera's picture is never still: someone talking moves a little all
//! the time, and how much depends on the person, the framing and the light.
//! So a movement is judged against the recording's own level, as the
//! screen's states are: a stretch where the share of the picture changing,
//! averaged over half a second, is several times the recording's median and
//! above a share that small movement never reaches. Movements a moment
//! apart are one: a gesture rarely moves the picture without pause, and a
//! string of them is one thing happening, not several.
//!
//! The whole frame is measured, so a change of light or the camera
//! adjusting its exposure counts too.

use crate::measure::PictureReading;

/// How many readings, at ten a second, each is averaged over.
const SMOOTH: usize = 5;
/// The least share of the picture that changes in a movement, in any
/// recording.
const LEAST_SHARE: f32 = 0.03;
/// How many times the recording's median change a movement is.
const ABOVE_MEDIAN: f32 = 4.0;
/// How long the picture must be calm, in milliseconds, between two
/// movements for them to be two.
const CALM_MS: u64 = 2_000;
/// The shortest movement kept, in milliseconds: a twitch, spread over the
/// half second each reading is averaged over, is shorter.
const LEAST_MS: u64 = 800;

/// A stretch of the recording, in its milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Movement {
  pub start_ms: u64,
  pub end_ms: u64,
}

pub fn movements(readings: &[PictureReading]) -> Vec<Movement> {
  if readings.is_empty() {
    return Vec::new();
  }
  let smoothed: Vec<f32> = (0..readings.len())
    .map(|index| {
      let around =
        &readings[index.saturating_sub(SMOOTH / 2)..(index + SMOOTH / 2 + 1).min(readings.len())];
      around.iter().map(|reading| reading.changed).sum::<f32>() / around.len() as f32
    })
    .collect();
  let mut sorted = smoothed.clone();
  sorted.sort_by(f32::total_cmp);
  let level = LEAST_SHARE.max(ABOVE_MEDIAN * sorted[sorted.len() / 2]);
  let mut found: Vec<Movement> = Vec::new();
  for (index, reading) in readings.iter().enumerate() {
    if smoothed[index] < level {
      continue;
    }
    let end_ms = readings
      .get(index + 1)
      .map_or(reading.ms + 100, |next| next.ms);
    match found.last_mut() {
      Some(last) if reading.ms <= last.end_ms + CALM_MS => last.end_ms = end_ms,
      _ => found.push(Movement {
        end_ms,
        start_ms: reading.ms,
      }),
    }
  }
  found.retain(|movement| movement.end_ms - movement.start_ms >= LEAST_MS);
  found
}
