// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Absolute-position layout tracks for keyboard visuals.

#[path = "layout/track_building.rs"]
mod track_building;
pub(super) use track_building::attach_tracks;

use std::collections::{BTreeMap, HashMap};

use super::{
  clock::AnimationClock,
  state::{TransitionKind, VisualKey},
};

pub(super) const MOTION_US: u64 = 600_000;
/// A key pressed this soon after the previous key of its chord is part of the
/// same rolled gesture, not a late join: the row is laid out for the whole
/// roll from its first key (each key still pops in on its own entrance)
/// instead of sliding the earlier keys aside.
const ASSEMBLY_US: u64 = 250_000;

#[derive(Clone, Debug, Default)]
pub(super) struct LayoutTrack {
  initial: Vec<u32>,
  changes: Vec<LayoutChange>,
  freeze_us: Option<u64>,
}

#[derive(Clone, Debug)]
struct LayoutChange {
  start_us: u64,
  from: Vec<u32>,
  to: Vec<u32>,
  start_progress: f32,
  end_progress: f32,
}

impl LayoutChange {
  fn settled(&self) -> &Vec<u32> {
    if self.end_progress < 0.5 {
      &self.from
    } else {
      &self.to
    }
  }
}

pub(super) struct LayoutSample<'a> {
  pub from: &'a [u32],
  pub to: &'a [u32],
  pub progress: f32,
}

#[derive(Default)]
struct Events {
  enters: Vec<usize>,
  freezes: Vec<usize>,
  removes: Vec<usize>,
}

impl LayoutTrack {
  fn new(initial: Vec<u32>) -> Self {
    Self {
      initial,
      ..Self::default()
    }
  }

  fn schedule(&mut self, at: u64, target: Vec<u32>) {
    self.changes.retain(|change| change.start_us < at);
    if let Some(change) = self.changes.last() {
      let elapsed = at.saturating_sub(change.start_us);
      if elapsed < MOTION_US {
        let fraction = elapsed as f32 / MOTION_US as f32;
        let current =
          change.start_progress + (change.end_progress - change.start_progress) * fraction;
        if target == change.to && change.end_progress == 1.0 {
          return;
        }
        if target == change.from {
          let from = change.from.clone();
          let to = change.to.clone();
          self.changes.push(LayoutChange {
            start_us: at,
            from,
            to,
            start_progress: current,
            end_progress: 0.0,
          });
          return;
        }
      }
    }
    let (from, available) = self.changes.last().map_or_else(
      || (self.initial.clone(), at),
      |change| {
        (
          change.settled().clone(),
          change.start_us.saturating_add(MOTION_US),
        )
      },
    );
    if from == target {
      return;
    }
    self.changes.push(LayoutChange {
      start_us: at.max(available),
      from,
      to: target,
      start_progress: 0.0,
      end_progress: 1.0,
    });
  }

  fn cancel_future(&mut self, at: u64) {
    self.changes.retain(|change| change.start_us < at);
  }

  fn available_at(&self, at: u64) -> u64 {
    self
      .changes
      .last()
      .filter(|change| change.start_us <= at && at.saturating_sub(change.start_us) < MOTION_US)
      .map_or(at, |change| change.start_us.saturating_add(MOTION_US))
  }

  fn freeze(&mut self, at: u64) {
    self.freeze_us = Some(self.freeze_us.map_or(at, |known| known.min(at)));
  }

  pub(super) fn sample(&self, now: u64, clock: AnimationClock<'_>) -> LayoutSample<'_> {
    let at = self.freeze_us.map_or(now, |freeze| now.min(freeze));
    let Some(change) = self
      .changes
      .iter()
      .rev()
      .find(|change| change.start_us <= at)
    else {
      return LayoutSample {
        from: &self.initial,
        to: &self.initial,
        progress: 1.0,
      };
    };
    let elapsed = clock.elapsed_us(change.start_us, at);
    if elapsed >= MOTION_US {
      let settled = change.settled();
      return LayoutSample {
        from: settled,
        to: settled,
        progress: 1.0,
      };
    }
    let fraction = elapsed as f32 / MOTION_US as f32;
    LayoutSample {
      from: &change.from,
      to: &change.to,
      progress: change.start_progress + (change.end_progress - change.start_progress) * fraction,
    }
  }

  fn representative_snapshot(&self, at: u64) -> Vec<u32> {
    let sample = self.sample(at, AnimationClock::source());
    if sample.progress < 0.5 {
      sample.from.to_vec()
    } else {
      sample.to.to_vec()
    }
  }
}

pub(super) fn mask(layout: &[u32], slots: &[u32]) -> u32 {
  layout.iter().fold(0, |mask, slot| {
    slots
      .iter()
      .position(|known| known == slot)
      .map_or(mask, |index| mask | (1 << index))
  })
}
