// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How loud a voice usually speaks: the energy of its speech, measured over
//! the stretches loud enough to be speech, so a pause or a breath does not
//! pull it down. The de-esser listens only while someone speaks by it, and
//! the cleanup gives back the loudness its tone shaping takes by it.

/// How long each frame the speech level is measured over is: 50 ms.
const FRAME_MS: usize = 50;
/// Frames this far below the loudest twentieth of them, 20 dB, are not
/// speech: pauses, breaths, the room.
const SPEECH_RANGE: f32 = 0.01;

/// The energy of each 50 ms of a track, gathered a run at a time.
pub(super) struct Frames {
  size: usize,
  sum: f64,
  count: usize,
  energies: Vec<f32>,
}

impl Frames {
  pub(super) fn new(rate: u32) -> Self {
    Self {
      size: rate as usize * FRAME_MS / 1_000,
      sum: 0.0,
      count: 0,
      energies: Vec::new(),
    }
  }

  pub(super) fn add(&mut self, samples: &[f32]) {
    for &sample in samples {
      self.sum += f64::from(sample * sample);
      self.count += 1;
      if self.count == self.size {
        self.energies.push((self.sum / self.size as f64) as f32);
        self.sum = 0.0;
        self.count = 0;
      }
    }
  }

  pub(super) fn energies(self) -> Vec<f32> {
    self.energies
  }
}

/// Which of the frames whose energies are `energies` hold speech, or
/// nothing if the track is silent.
pub(super) fn speaking(energies: &[f32]) -> Option<Vec<bool>> {
  let mut sorted = energies.to_vec();
  let loud = sorted.len().checked_sub(1)? * 95 / 100;
  let (_, &mut reference, _) = sorted.select_nth_unstable_by(loud, f32::total_cmp);
  (reference > 1e-10).then(|| {
    energies
      .iter()
      .map(|&energy| energy > reference * SPEECH_RANGE)
      .collect()
  })
}

/// The average energy of the frames `speaking` marks.
pub(super) fn speech_energy(energies: &[f32], speaking: &[bool]) -> f32 {
  let (sum, count) = energies
    .iter()
    .zip(speaking)
    .filter(|(_, &speaking)| speaking)
    .fold((0.0_f64, 0_usize), |(sum, count), (&energy, _)| {
      (sum + f64::from(energy), count + 1)
    });
  (sum / count.max(1) as f64) as f32
}
