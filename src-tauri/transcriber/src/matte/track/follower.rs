// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The objects followed one way through the recording, a frame in
//! [`STRIDE`] at a time where they sit still, every frame where they move.

use super::cutie::{Cutie, Object, MASK_HEIGHT, MASK_WIDTH, WIDTH};
use super::slots::Slots;

/// One frame in how many is followed where nothing moves.
const STRIDE: usize = 4;
/// How much of an object's mask may change between two followed frames,
/// as a share of the two together, before the frames between are followed.
const MOVED: f32 = 0.05;
/// How long every object may be gone from the frame before following them
/// that way stops: long enough for one to come back from behind a passing
/// hand, short of following an empty room, or of a mask that has lost its
/// object wandering onto another.
pub(super) const GONE_MS: f64 = 2_000.0;
/// How sure a mask cell must be of its object to count it there.
const THERE: f32 = 0.5;

/// The objects followed one way through the recording, frame by frame as
/// they are pushed, a frame in [`STRIDE`] at a time.
pub(super) struct Follower {
  objects: Vec<Object>,
  /// Each object's mask in the frame last followed.
  masks: Vec<Vec<f32>>,
  waiting: Vec<(usize, Vec<u8>)>,
  /// The frame the objects were last seen in, of those followed.
  seen: Option<usize>,
}

impl Follower {
  pub(super) fn new(objects: Vec<Object>, masks: Vec<Vec<f32>>) -> Self {
    Self {
      objects,
      masks,
      waiting: Vec::with_capacity(STRIDE),
      seen: None,
    }
  }

  /// Whether every object has been gone from the frame since longer than
  /// [`GONE_MS`] before the one at `ms`, of `frames` timed in milliseconds.
  pub(super) fn gone(&self, frames: &[f64], ms: f64) -> bool {
    self
      .seen
      .is_some_and(|seen| (ms - frames[seen]).abs() > GONE_MS)
  }

  pub(super) fn push(
    &mut self,
    cutie: &mut Cutie,
    slots: &mut Slots,
    index: usize,
    rgb: Vec<u8>,
    tell: &mut dyn FnMut(usize),
  ) -> Result<(), String> {
    self.waiting.push((index, rgb));
    if self.waiting.len() == STRIDE {
      self.follow(cutie, slots, tell)?;
    }
    Ok(())
  }

  pub(super) fn finish(
    &mut self,
    cutie: &mut Cutie,
    slots: &mut Slots,
    tell: &mut dyn FnMut(usize),
  ) -> Result<(), String> {
    if self.waiting.is_empty() {
      return Ok(());
    }
    self.follow(cutie, slots, tell)
  }

  /// Follows the last frame waiting, then each object through the frames
  /// before it: followed where it moved, blended where it did not.
  fn follow(
    &mut self,
    cutie: &mut Cutie,
    slots: &mut Slots,
    tell: &mut dyn FnMut(usize),
  ) -> Result<(), String> {
    let waiting = std::mem::take(&mut self.waiting);
    let Some(((last, rgb), between)) = waiting.split_last() else {
      return Ok(());
    };
    let before = self.objects.clone();
    let seen = cutie.see(rgb)?;
    let mut moved = Vec::new();
    for (number, object) in self.objects.iter_mut().enumerate() {
      let mask = cutie.find(&seen, object, true)?;
      if !between.is_empty() && change(&self.masks[number], &mask) > MOVED {
        moved.push(number);
      } else {
        for (step, (index, _)) in between.iter().enumerate() {
          let share = (step + 1) as f32 / waiting.len() as f32;
          let blended = self.masks[number]
            .iter()
            .zip(&mask)
            .map(|(&from, &to)| from * (1.0 - share) + to * share)
            .collect::<Vec<_>>();
          slots.write(number, *index, &blended)?;
        }
      }
      slots.write(number, *last, &mask)?;
      self.masks[number] = mask;
    }
    if self.seen.is_none()
      || self
        .masks
        .iter()
        .any(|mask| mask.iter().any(|&value| value > THERE))
    {
      self.seen = Some(*last);
    }
    if !moved.is_empty() {
      let mut filling = moved
        .iter()
        .map(|&number| before[number].clone())
        .collect::<Vec<_>>();
      for (index, rgb) in between {
        let seen = cutie.see(rgb)?;
        for (object, &number) in filling.iter_mut().zip(&moved) {
          let mask = cutie.find(&seen, object, false)?;
          slots.write(number, *index, &mask)?;
        }
      }
    }
    tell(waiting.len());
    Ok(())
  }
}

/// How much two masks differ: what is in one and not the other, as a share
/// of what is in either.
fn change(from: &[f32], to: &[f32]) -> f32 {
  let (mut either, mut one) = (0_usize, 0_usize);
  for (&a, &b) in from.iter().zip(to) {
    let (a, b) = (a > 0.5, b > 0.5);
    either += usize::from(a || b);
    one += usize::from(a != b);
  }
  one as f32 / either.max(1) as f32
}

/// A full-size probability at a quarter of the size, each mask pixel the
/// mean of the sixteen it covers.
pub(super) fn quarter(full: &[f32]) -> Vec<f32> {
  let mut mask = vec![0.0_f32; MASK_WIDTH * MASK_HEIGHT];
  for (index, &value) in full.iter().enumerate() {
    let (x, y) = (index % WIDTH, index / WIDTH);
    mask[(y / 4) * MASK_WIDTH + x / 4] += value / 16.0;
  }
  mask
}
