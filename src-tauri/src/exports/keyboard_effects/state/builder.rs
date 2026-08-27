// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Physical-event builder for deterministic keyboard-chord display state.

use std::collections::HashMap;

use super::{role, KeyboardStateTimeline, LayoutTrack, TransitionKind, VisualKey, VisualRole};
use crate::exports::keyboard_effects::clock::AnimationClock;
use crate::exports::keyboard_effects::HOLD_US;
use crate::exports::keyboard_effects::{KeyPress, Shortcut};
use crate::exports::timeline_edit::{
  source_before_output_duration_us, DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange,
  TimelineRange,
};

/// The timeline-edit state a chord's badge continuity depends on. A chord
/// continues its predecessor's badge only when both are visible then and
/// occupy the same place at the same size.
#[derive(Clone, Copy, Default)]
pub(in crate::exports::keyboard_effects) struct ChainContext<'a> {
  pub deleted_ids: &'a [u64],
  pub deleted_ranges: &'a [DeletedKeyboardShortcutRange],
  pub positions: &'a [KeyboardShortcutPositionRange],
  /// The edit's retained playback ranges; animation durations run on the
  /// output clock, so bake-time scheduling maps through them.
  pub ranges: Option<&'a [TimelineRange]>,
}

impl<'a> ChainContext<'a> {
  pub(super) fn clock(&self) -> AnimationClock<'a> {
    self
      .ranges
      .map_or_else(AnimationClock::source, AnimationClock::edited)
  }

  /// The source instant at which an exit fade must start so it has fully
  /// played out - in output time - by `press_us`.
  fn fade_finished_by(&self, press_us: u64) -> u64 {
    source_before_output_duration_us(self.ranges, press_us, super::EXIT_US)
      .unwrap_or_else(|| press_us.saturating_sub(super::EXIT_US))
  }
}

impl ChainContext<'_> {
  fn deleted(&self, shortcut: usize, at_ms: u64) -> bool {
    let shortcut = shortcut as u64;
    self.deleted_ids.contains(&shortcut)
      || self.deleted_ranges.iter().any(|range| {
        range.shortcut_id == shortcut && at_ms >= range.start_ms && at_ms < range.end_ms
      })
  }

  fn placement(&self, shortcut: usize, at_ms: u64) -> Option<(f64, f64, Option<f64>)> {
    self
      .positions
      .iter()
      .find(|position| {
        position.shortcut_id == shortcut as u64
          && at_ms >= position.start_ms
          && at_ms < position.end_ms
      })
      .map(|position| (position.center_x, position.center_y, position.size_percent))
  }

  fn deleted_at(&self, shortcut: usize, at_us: u64) -> bool {
    self.deleted(shortcut, at_us / 1_000)
  }

  fn chains(&self, previous: usize, next: usize, at_us: u64) -> bool {
    let at_ms = at_us / 1_000;
    if self.deleted(previous, at_ms) || self.deleted(next, at_ms) {
      return false;
    }
    self.placement(previous, at_ms) == self.placement(next, at_ms)
  }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalEvent {
  shortcut: usize,
  at: u64,
  key_code: u16,
  modifier_mask: u32,
  down: bool,
}

#[derive(Clone, Copy, Debug)]
struct Slot {
  id: u32,
  group: u32,
  role: VisualRole,
  current: Option<usize>,
}

#[derive(Default)]
struct Builder<'a> {
  context: ChainContext<'a>,
  timeline: KeyboardStateTimeline,
  slots: Vec<Slot>,
  held: HashMap<u16, usize>,
  next_slot_id: u32,
  current_group: u32,
}

impl KeyboardStateTimeline {
  pub(in crate::exports::keyboard_effects) fn from_shortcuts(
    shortcuts: &[Shortcut],
    context: ChainContext<'_>,
  ) -> Self {
    let mut events = shortcuts
      .iter()
      .enumerate()
      .flat_map(|(shortcut, shortcut_data)| {
        shortcut_data
          .keys
          .iter()
          .flat_map(move |press| events_for_press(shortcut, press))
      })
      .flatten()
      .collect::<Vec<_>>();
    events.sort_by_key(|event| (event.at, event.down));
    let mut builder = Builder {
      context,
      ..Builder::default()
    };
    for event in events {
      if event.down {
        builder.key_down(event);
      } else {
        builder.key_up(event);
      }
    }
    builder.timeline
  }
}

fn events_for_press(shortcut: usize, press: &KeyPress) -> [Option<PhysicalEvent>; 2] {
  [
    Some(PhysicalEvent {
      shortcut,
      at: press.down_us,
      key_code: press.key_code,
      modifier_mask: press.modifier_mask,
      down: true,
    }),
    press.up_us.map(|at| PhysicalEvent {
      shortcut,
      at,
      key_code: press.key_code,
      modifier_mask: press.modifier_mask,
      down: false,
    }),
  ]
}

impl Builder<'_> {
  fn key_down(&mut self, event: PhysicalEvent) {
    self.clear_finished(event.at);
    if self.held.contains_key(&event.key_code) {
      return;
    }
    let role = role(event.key_code);
    let new_physical_chord = self.held.is_empty();
    let mut reusable_slot = None;
    let mut replacement_enter = false;
    let mut replacement_candidates = Vec::new();
    if new_physical_chord {
      // The new chord continues the badge on screen only when it will draw
      // in the same place at the same size; otherwise that badge finishes
      // the exit it already has scheduled, completely untouched, and this
      // chord starts a fresh badge with fresh slots.
      let predecessor = self
        .slots
        .iter()
        .filter_map(|slot| slot.current)
        .max_by_key(|visual| self.timeline.visuals[*visual].enter_us);
      // A badge that has begun its exit fade is already leaving, so a new
      // chord never continues it: the fade is finished by this press below
      // and the chord pops in as a fresh badge instead.
      let chains = predecessor.is_some_and(|visual| {
        let known = &self.timeline.visuals[visual];
        known.exit.is_none_or(|(exit_us, _)| event.at < exit_us)
          && self
            .context
            .chains(known.source_shortcut, event.shortcut, event.at)
      });
      if chains {
        for index in 0..self.slots.len() {
          if self.slots[index].group != self.current_group {
            continue;
          }
          if let Some(visual) = self.slots[index].current.take() {
            self.start_exit(visual, event.at, TransitionKind::Replacement);
            replacement_candidates.push((self.slots[index].id, visual));
          }
        }
      } else {
        self.current_group += 1;
        // Every key is physically released by now, so lingering badges are in
        // their post-release hold. A new visible badge makes that hold a lie
        // (the same key may already be down again), so the fade is pulled
        // forward far enough to be FINISHED by this press - while never
        // starting before the badge's own keys were actually released.
        if !self.context.deleted_at(event.shortcut, event.at) {
          let fade_start = self.context.fade_finished_by(event.at);
          let lingering = self
            .slots
            .iter()
            .filter_map(|slot| slot.current)
            .collect::<Vec<_>>();
          for visual in lingering {
            let visual = &mut self.timeline.visuals[visual];
            if let Some((exit_us, kind)) = visual.exit {
              let release_us = exit_us.saturating_sub(HOLD_US);
              let finished_by_press = fade_start.max(release_us);
              visual.exit = Some((exit_us.min(finished_by_press), kind));
            }
          }
        }
      }
    } else {
      replacement_candidates = self.retire_released(event.at);
      reusable_slot = replacement_candidates.iter().find_map(|(slot, _)| {
        self
          .slot(*slot)
          .is_some_and(|known| known.role.same_slot_kind(role))
          .then_some(*slot)
      });
      if role == VisualRole::Primary {
        if let Some(slot) = self
          .slots
          .iter()
          .find(|slot| {
            slot.group == self.current_group
              && slot.role == VisualRole::Primary
              && slot.current.is_some()
          })
          .map(|slot| slot.id)
        {
          if let Some(visual) = self.slot_mut(slot).and_then(|known| known.current.take()) {
            self.start_exit(visual, event.at, TransitionKind::Replacement);
          }
          reusable_slot = Some(slot);
          replacement_enter = true;
        }
      }
    }
    reusable_slot = reusable_slot.or_else(|| {
      replacement_candidates.iter().find_map(|(slot, _)| {
        self
          .slot(*slot)
          .is_some_and(|known| known.role.same_slot_kind(role))
          .then_some(*slot)
      })
    });
    if reusable_slot.is_some_and(|slot| {
      replacement_candidates
        .iter()
        .any(|(candidate, _)| *candidate == slot)
    }) {
      replacement_enter = true;
    }
    if reusable_slot.is_none() {
      let inactive = self
        .slots
        .iter()
        .filter(|slot| {
          slot.group == self.current_group
            && slot.current.is_none()
            && slot.role.same_slot_kind(role)
        })
        .map(|slot| (slot.id, self.latest_visible_in_slot(slot.id, event.at)))
        .min_by_key(|(slot, visible)| (visible.is_none(), *slot));
      if let Some((slot, outgoing)) = inactive {
        reusable_slot = Some(slot);
        replacement_enter = outgoing.is_some();
        if let Some(outgoing) = outgoing {
          self.start_exit(outgoing, event.at, TransitionKind::Replacement);
        }
      }
    }
    if let Some(slot_id) = reusable_slot {
      replacement_candidates.retain(|(candidate, _)| *candidate != slot_id);
    }
    let slot_id = if let Some(slot_id) = reusable_slot {
      if let Some(slot) = self.slot_mut(slot_id) {
        slot.role = role;
      }
      slot_id
    } else {
      let slot_id = self.next_slot_id;
      self.next_slot_id += 1;
      self.slots.push(Slot {
        id: slot_id,
        group: self.current_group,
        role,
        current: None,
      });
      slot_id
    };
    let visual = self.timeline.visuals.len();
    self.timeline.visuals.push(VisualKey {
      source_shortcut: event.shortcut,
      group: self.current_group,
      key_code: event.key_code,
      modifier_mask: event.modifier_mask,
      role,
      slot_id,
      enter_us: event.at,
      animation_enter_us: event.at,
      reserve_from_us: None,
      replacement_enter,
      layout_exit_us: None,
      layout_anchor_until_us: None,
      exit: None,
      layout: LayoutTrack::default(),
    });
    self
      .slot_mut(slot_id)
      .expect("new slots remain present")
      .current = Some(visual);
    self.held.insert(event.key_code, visual);
    for (_, outgoing) in replacement_candidates {
      let visual = &mut self.timeline.visuals[outgoing];
      visual.layout_exit_us = Some(event.at);
      if let Some((exit_us, _)) = visual.exit {
        visual.exit = Some((exit_us, TransitionKind::Detached));
      }
    }
  }

  fn key_up(&mut self, event: PhysicalEvent) {
    self.clear_finished(event.at);
    let Some(visual) = self.held.remove(&event.key_code) else {
      return;
    };
    if self.held.is_empty() {
      let deadline = event.at.saturating_add(HOLD_US);
      let currents = self
        .slots
        .iter()
        .filter(|slot| slot.group == self.current_group)
        .filter_map(|slot| slot.current)
        .collect::<Vec<_>>();
      for current in currents {
        self.schedule_release(current, deadline, TransitionKind::GroupRelease);
      }
    } else {
      self.schedule_release(
        visual,
        event.at.saturating_add(HOLD_US),
        TransitionKind::Release,
      );
    }
  }

  fn retire_released(&mut self, at: u64) -> Vec<(u32, usize)> {
    let held = self.held.values().copied().collect::<Vec<_>>();
    let released = self
      .slots
      .iter()
      .filter(|slot| slot.group == self.current_group)
      .filter_map(|slot| {
        slot
          .current
          .filter(|visual| !held.contains(visual))
          .map(|visual| (slot.id, visual))
      })
      .collect::<Vec<_>>();
    for (slot_id, visual) in &released {
      self.start_exit(*visual, at, TransitionKind::Replacement);
      if let Some(slot) = self.slot_mut(*slot_id) {
        slot.current = None;
      }
    }
    released
  }

  fn schedule_release(&mut self, visual: usize, deadline: u64, kind: TransitionKind) {
    if self.slots.iter().any(|slot| slot.current == Some(visual)) {
      self.timeline.visuals[visual].exit = Some((deadline, kind));
      if kind == TransitionKind::Release {
        self.timeline.visuals[visual].layout_exit_us = Some(deadline);
      } else if kind == TransitionKind::GroupRelease {
        self.timeline.visuals[visual].layout_exit_us = None;
      }
    }
  }

  fn start_exit(&mut self, visual: usize, at: u64, kind: TransitionKind) {
    let known = &mut self.timeline.visuals[visual];
    let exit_at = known.exit.map_or(at, |(scheduled, _)| scheduled.min(at));
    known.exit = Some((exit_at, kind));
    if kind == TransitionKind::Replacement {
      known.layout_exit_us = None;
    }
  }

  fn clear_finished(&mut self, at: u64) {
    let clock = self.context.clock();
    let finished = self
      .slots
      .iter()
      .filter_map(|slot| {
        let visual = slot.current?;
        (!self.timeline.visuals[visual].visible_at(at, clock)).then_some(slot.id)
      })
      .collect::<Vec<_>>();
    for slot_id in finished {
      if let Some(slot) = self.slot_mut(slot_id) {
        slot.current = None;
      }
    }
  }

  fn slot(&self, id: u32) -> Option<&Slot> {
    self.slots.iter().find(|slot| slot.id == id)
  }

  fn slot_mut(&mut self, id: u32) -> Option<&mut Slot> {
    self.slots.iter_mut().find(|slot| slot.id == id)
  }

  fn latest_visible_in_slot(&self, slot: u32, at: u64) -> Option<usize> {
    self
      .timeline
      .visuals
      .iter()
      .enumerate()
      .filter(|(_, visual)| visual.slot_id == slot && visual.visible_at(at, self.context.clock()))
      .max_by_key(|(_, visual)| visual.enter_us)
      .map(|(index, _)| index)
  }
}
