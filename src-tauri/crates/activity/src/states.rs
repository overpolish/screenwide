// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the picture was doing, judged against the recording's own level.
//!
//! The same share of change means different things in different
//! recordings. On a screen recording half the readings change nothing, and
//! a tenth of the picture going new is another page. In a first-person game
//! the median reading changes over a third of the picture and is over half
//! new, every turn of the view; what stands out there is the reverse: the
//! death screen, the scoreboard, the loading screen, where the picture stops.
//! So a reading is moving against half the recording's median change, and
//! new where it is twice as new as the readings around it. In a recording
//! that is new much of the time, as play in a game is, it must also be
//! among the newest few in it, or every quick pan of the view would count.
//! A screen recording is not held to that: a video it shows would make its
//! newest few all video, and a window opening beside one go unseen.
//!
//! A new screen also settles: the second after it is calm. Scrolling brings
//! new content every tenth of a second without settling until it stops, so
//! it reads as moving, and where it stops, on content not seen a moment
//! before, as new.

use crate::measure::{PictureReading, PictureState};

/// The least share of the picture that moves, and is new, in any recording.
const MOVING_SHARE: f32 = 0.002;
const NEW_SHARE: f32 = 0.25;
/// How new a recording's median reading is, past which it is busy.
const BUSY_NOVELTY: f32 = 0.05;
/// In a busy recording, how rare a new reading is: the share above it.
const NEW_RARITY: f32 = 0.03;
/// How many readings around one are its surroundings: ten seconds.
const AROUND: usize = 100;
/// How many readings after a new one must be calm: a second.
const SETTLE: usize = 10;
/// The least share of the picture that changes while it is not calm.
const CALM_SHARE: f32 = 0.05;

pub fn states(readings: &[PictureReading]) -> Vec<PictureState> {
  if readings.is_empty() {
    return Vec::new();
  }
  let changed: Vec<f32> = readings.iter().map(|reading| reading.changed).collect();
  let novelty: Vec<f32> = readings.iter().map(|reading| reading.novelty).collect();
  let moving = MOVING_SHARE.max(0.5 * quantile(&changed, 0.5));
  let new = if quantile(&novelty, 0.5) > BUSY_NOVELTY {
    NEW_SHARE.max(quantile(&novelty, 1.0 - NEW_RARITY))
  } else {
    NEW_SHARE
  };
  readings
    .iter()
    .enumerate()
    .map(|(index, reading)| {
      let from = index.saturating_sub(AROUND / 2);
      let to = (index + AROUND / 2).min(readings.len());
      let around = quantile(&novelty[from..to], 0.5);
      let next = &changed[(index + 1).min(changed.len())..(index + 1 + SETTLE).min(changed.len())];
      let settles = next.iter().sum::<f32>() / next.len().max(1) as f32 <= CALM_SHARE.max(moving);
      if reading.novelty >= new && reading.novelty >= 2.0 * around && settles {
        PictureState::New
      } else if reading.changed >= moving {
        PictureState::Moving
      } else {
        PictureState::Still
      }
    })
    .collect()
}

/// The value `share` of the way up `values` sorted.
fn quantile(values: &[f32], share: f32) -> f32 {
  let mut sorted = values.to_vec();
  sorted.sort_by(f32::total_cmp);
  let index = ((sorted.len() - 1) as f32 * share).round() as usize;
  sorted[index]
}
