// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Absolute-position layout tracks for keyboard visuals.

use std::collections::{BTreeMap, HashMap};

use super::state::{TransitionKind, VisualKey};

const MOTION_US: u64 = 600_000;

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

  pub(super) fn sample(&self, now: u64) -> LayoutSample<'_> {
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
    let elapsed = at.saturating_sub(change.start_us);
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
    let sample = self.sample(at);
    if sample.progress < 0.5 {
      sample.from.to_vec()
    } else {
      sample.to.to_vec()
    }
  }
}

pub(super) fn attach_tracks(visuals: &mut [VisualKey]) {
  let mut events = BTreeMap::<u64, Events>::new();
  for (index, visual) in visuals.iter().enumerate() {
    events
      .entry(visual.enter_us)
      .or_default()
      .enters
      .push(index);
    if let Some((at, kind)) = visual.exit {
      if kind != TransitionKind::Replacement {
        events.entry(at).or_default().freezes.push(index);
      }
      let remove_at = if kind == TransitionKind::Detached {
        visual
          .layout_exit_us
          .unwrap_or_else(|| at.saturating_add(MOTION_US))
      } else {
        at.saturating_add(MOTION_US)
      };
      events.entry(remove_at).or_default().removes.push(index);
    }
  }
  let mut active = HashMap::<u32, usize>::new();
  for (at, event) in events {
    for &index in &event.freezes {
      visuals[index].layout.freeze(at);
    }
    let mut removed = Vec::new();
    for &index in &event.removes {
      if active.get(&visuals[index].slot_id) == Some(&index) {
        active.remove(&visuals[index].slot_id);
        removed.push(index);
      }
    }
    let mut existing = tracked_indices(visuals, &active, at);
    for &index in &existing {
      visuals[index].layout.cancel_future(at);
    }
    let barrier = existing.iter().fold(at, |available, index| {
      available.max(visuals[*index].layout.available_at(at))
    });
    let mut base = existing.first().map_or_else(
      || ordered_slots(visuals, active.values().copied()),
      |index| visuals[*index].layout.representative_snapshot(barrier),
    );
    let entering = event.enters.to_vec();
    for &index in &entering {
      active.insert(visuals[index].slot_id, index);
    }
    let target = ordered_slots(visuals, active.values().copied());
    if base.is_empty() && removed.is_empty() {
      base.clone_from(&target);
    }
    for &index in &entering {
      let replacement = if visuals[index].replacement_enter {
        replacement_track(visuals, index, at)
      } else {
        None
      };
      visuals[index].layout = replacement.unwrap_or_else(|| LayoutTrack::new(base.clone()));
    }
    existing = tracked_indices(visuals, &active, at);
    for &index in &existing {
      visuals[index].layout.cancel_future(at);
      if !base.contains(&visuals[index].slot_id) {
        visuals[index].animation_enter_us = barrier;
      }
    }
    for &index in &removed {
      if visuals[index]
        .exit
        .is_some_and(|(_, kind)| kind == TransitionKind::Detached)
      {
        visuals[index].exit = Some((barrier, TransitionKind::Detached));
      }
      visuals[index].layout.freeze(barrier);
      if !active.is_empty() {
        visuals[index].layout_anchor_until_us = Some(barrier.saturating_add(MOTION_US));
      }
    }
    for index in existing {
      visuals[index].layout.schedule(barrier, target.clone());
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

fn ordered_slots(visuals: &[VisualKey], indices: impl Iterator<Item = usize>) -> Vec<u32> {
  let mut indices = indices.collect::<Vec<_>>();
  indices.sort_by_key(|index| (visuals[*index].role.order(), visuals[*index].slot_id));
  indices
    .into_iter()
    .map(|index| visuals[index].slot_id)
    .collect()
}

fn replacement_track(visuals: &[VisualKey], incoming: usize, at: u64) -> Option<LayoutTrack> {
  visuals
    .iter()
    .enumerate()
    .filter(|(index, visual)| {
      *index != incoming
        && visual.slot_id == visuals[incoming].slot_id
        && visual.enter_us < at
        && visual.visible_at(at)
    })
    .max_by_key(|(_, visual)| visual.enter_us)
    .map(|(_, visual)| visual.layout.clone())
}

fn tracked_indices(visuals: &[VisualKey], active: &HashMap<u32, usize>, at: u64) -> Vec<usize> {
  let mut tracked = active.values().copied().collect::<Vec<_>>();
  tracked.extend(visuals.iter().enumerate().filter_map(|(index, visual)| {
    visual
      .exit
      .is_some_and(|(exit_us, kind)| {
        kind == TransitionKind::Replacement
          && exit_us <= at
          && visual.visible_at(at)
          && active.contains_key(&visual.slot_id)
      })
      .then_some(index)
  }));
  tracked.sort_unstable();
  tracked.dedup();
  tracked
}
